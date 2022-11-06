use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service, post},
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use lib_device::*;
use serde::Deserialize;
// use serde_json::*;
use std::time::Instant;

use std::{
    collections::HashSet,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tower_http::services::ServeDir;
use uuid::Uuid;

use colored::*;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DB_URI: &str = "sqlite://data.db";
// Duration in seconds between two separate logs.
const LOG_RATE: u64 = 1;

#[derive(Clone, Deserialize)]
struct DeviceData {
    devices: Vec<Device>,
}
#[derive(Clone, Deserialize)]
struct ExportOptions;

#[derive(Clone)]
struct Msg {
    client_id: String,
    payload: String,
}
// The shared State
struct AppState {
    // A hashset to keep track of our connected clients.
    client_set: Mutex<HashSet<String>>,
    // A channel sender contex that we use to communicate with the threads.
    tx: broadcast::Sender<Msg>,
    // An SQLITE connection pool that we use to execute queries on the database.
    db_pool: SqlitePool,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "prussian_server=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client_set: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

    // We check if the database file exists, otherwise we create it.
    if !Sqlite::database_exists(DB_URI).await.unwrap_or(false) {
        Sqlite::create_database(DB_URI).await.unwrap();
    }

    // We establish a connection pool with the database.
    let db_pool = SqlitePoolOptions::new()
        .connect(DB_URI)
        .await
        // Panic on error with the message:
        .expect("Something went wrong...");

    // We create the data tables inside the database if they don't exist.
    let query = r#"
    CREATE TABLE IF NOT EXISTS Records (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        datetime INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS Data (
        data_id INTEGER PRIMARY KEY AUTOINCREMENT,
        channel_id int NOT NULL,
        device_id int NOT NULL,
        value FLOAT(14, 4) NOT NULL,
        record_id INTEGER,
        FOREIGN KEY (record_id)
            REFERENCES Records(id)
    );"#;

    let result = sqlx::query(&query).execute(&db_pool).await.unwrap();
    println!("{:?}", result);

    // We create the channel that we will use to transfer messages between clients.
    let (tx, _rx) = broadcast::channel(3);

    // and innitiate our AppState.
    let app_state = Arc::new(AppState {
        client_set,
        tx,
        db_pool,
    });

    let hmi_dir = PathBuf::from(".").join("assets").join("HMI");
    let logger_dir = PathBuf::from(".").join("assets").join("logger");

    // We serve the HMI as ServeDir service.
    let hmi_service = ServeDir::new(hmi_dir).append_index_html_on_directories(true);
    let logger_service = ServeDir::new(logger_dir).append_index_html_on_directories(true);

    let app = Router::with_state(app_state)
        .route("/websocket", get(ws_handler))
        .route("/export", post(fetch_data))
        .nest(
            "/hmi/",
            get_service(hmi_service.clone()).handle_error(handle_error),
        )
        .nest("/", get_service(hmi_service).handle_error(handle_error))
        .nest(
            "/logger/",
            get_service(logger_service).handle_error(handle_error),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    if cfg!(target_os = "windows") {
        println!("{}", "Server is running...");
        println!("    => {} {}", "address:", &addr.ip().to_string());
        println!("    => {} {}", "port:", &addr.port().to_string());
        println!(
            "    => {} http://{}:{}",
            "link:",
            &addr.ip().to_string(),
            &addr.port().to_string(),
        );
    } else {
        println!("{}", "Server is running...".bold());
        println!(
            "    => {} {}",
            "address:".cyan(),
            &addr.ip().to_string().bold()
        );
        println!(
            "    => {} {}",
            "port:".cyan(),
            &addr.port().to_string().bold()
        );
        println!(
            "    => {} http://{}:{}",
            "link:".cyan(),
            &addr.ip().to_string().bold(),
            &addr.port().to_string().bold(),
        );
    }

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();

    ws.on_upgrade(|socket| websocket(socket, id, state))
}

async fn websocket(stream: WebSocket, id: String, state: Arc<AppState>) {
    //We check if the client is registered. We add it to the client set if not.

    if !is_registered(&state, &id) {
        if cfg!(target_os = "windows") {
            println!(
                "{} => {} is now registered.",
                chrono::Local::now().to_string(),
                &id
            );
        } else {
            println!(
                "{} => {} is now registered.",
                chrono::Local::now().to_string().dimmed().bold(),
                &id.green().dimmed().bold()
            );
        }
    }

    // We split the stream to be able to receive and send.

    let (mut sender, mut receiver) = stream.split();

    // We subscribe to the channel.

    let mut rx = state.tx.subscribe();

    // We spawn a task that receives any broadcasted messages and send them to our client.

    let client_id = id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // We check if the msg is coming from this same client. We don't want to send the msg to ourselves.
            if msg.client_id != client_id {
                if sender.send(Message::Text(msg.payload)).await.is_err() {
                    break;
                }
            }
        }
    });

    // We spawn a task that receives the socket msgs from our client and broadcasts them to other clients.
    let tx = state.tx.clone();
    let client_id = id.clone();
    let db_pool_cloned = state.db_pool.clone();

    let mut receive_task = tokio::spawn(async move {
        // We use a timer
        let mut time = Instant::now();
        while let Some(Ok(payload)) = receiver.next().await {
            match payload {
                Message::Binary(payload) => {
                    let text = String::from_utf8_lossy(&payload);
                    let msg = Msg {
                        client_id: client_id.clone(),
                        payload: text.to_string(),
                    };

                    let _ = tx.send(msg);
                }
                Message::Text(text) => {
                    let msg = Msg {
                        client_id: client_id.clone(),
                        payload: text,
                    };
                    // We log the data to the database.
                    if let Ok(devices_data) = serde_json::from_str(&msg.payload) {
                        if time.elapsed().as_secs() >= LOG_RATE {
                            let devices_to_log: DeviceData = devices_data;
                            log_data(&db_pool_cloned, &devices_to_log).await;
                            // And we reset the timer.
                            time = Instant::now();
                        }
                    }
                    let _ = tx.send(msg);
                }
                _ => {}
            }
        }
    });

    // We abort the tasks if anyone exits.

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    if cfg!(target_os = "windows") {
        println!(
            "{} => {} left the session.",
            chrono::Local::now().to_string(),
            &id
        );
    } else {
        println!(
            "{} => {} left the session.",
            chrono::Local::now().to_string().dimmed().bold(),
            &id.green().dimmed().bold()
        );
    }
    // Remove client from map.
    if let Ok(mut client_handle) = state.client_set.lock() {
        client_handle.remove(&id);
    }
}

fn is_registered(state: &AppState, id: &str) -> bool {
    let mut client_ids = state.client_set.lock().unwrap();

    if !client_ids.contains(id) {
        client_ids.insert(id.to_owned());
        false
    } else {
        true
    }
}

async fn log_data(db_pool: &SqlitePool, data: &DeviceData) {
    let datetime = chrono::Local::now();
    let timestamp = datetime.timestamp();
    let record_query = "INSERT INTO records (id, datetime) VALUES(NULL, $1)";
    let result = sqlx::query(&record_query)
        .bind(timestamp)
        .execute(db_pool)
        .await
        .unwrap();
    let row_id = &result.last_insert_rowid();

    let data_query = "INSERT INTO data (data_id, channel_id, device_id, value, record_id) 
                                    VALUES(NULL, $1, $2, $3, $4)";
    for device in &data.devices {
        for channel in &device.channels {
            let _result = sqlx::query(&data_query)
                .bind(channel.id as i32)
                .bind(channel.device_id as i32)
                .bind(channel.value)
                .bind(row_id)
                .execute(db_pool)
                .await
                .unwrap();
            //println!("{:?}", &result);
        }
    }
}

async fn fetch_data(
    Json(payload): Json<ExportOptions>,
    // State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
}
