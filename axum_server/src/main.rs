use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashSet,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tower_http::services::ServeDir;
use uuid::Uuid;

use colored::*;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// use rusqlite::{Connection, Result};

#[derive(Clone)]
struct Msg {
    client_id: String,
    payload: String,
}
// The shared State
struct AppState {
    client_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<Msg>,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_chat=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client_set: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

    // We create the channel that we will use to transfer messages between clients.
    let (tx, _rx) = broadcast::channel(3);

    let app_state = Arc::new(AppState { client_set, tx });

    let hmi_dir = PathBuf::from(".").join("assets").join("HMI");
    let logger_dir = PathBuf::from(".").join("assets").join("logger");

    let hmi_service = ServeDir::new(hmi_dir).append_index_html_on_directories(true);
    let logger_service = ServeDir::new(logger_dir).append_index_html_on_directories(true);

    let app = Router::with_state(app_state)
        .route("/websocket", get(ws_handler))
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

    let mut receive_task = tokio::spawn(async move {
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
    state.client_set.lock().unwrap().remove(&id);
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
