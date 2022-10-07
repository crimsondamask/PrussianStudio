use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let app = Router::with_state(app_state).route("/websocket", get(ws_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    ws.on_upgrade(|socket| websocket(socket, id, state))
}

async fn websocket(stream: WebSocket, id: String, state: Arc<AppState>) {
    //We check if the client is registered. We add it to the client set if not.

    if !is_registered(&state, &id) {
        println!("{} is now registered!", &id);
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

    let msg = format!("{} left.", &id);
    tracing::debug!("{}", msg);
    println!("{}", msg);
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
