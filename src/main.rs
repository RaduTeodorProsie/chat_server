use axum::{
    Router,
    extract::Path,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

type Room = broadcast::Sender<(Uuid, String)>;
type StateType = Arc<RwLock<HashMap<String, Room>>>;

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path((room, nickname)): Path<(String, String)>,
    State(state): State<StateType>,
) -> impl IntoResponse {
    let room_sender = {
        let mut rooms = state.write().unwrap();
        rooms
            .entry(room.clone())
            .or_insert_with(|| broadcast::channel(50).0)
            .clone()
    };
    ws.on_upgrade(move |socket| handle_socket(socket, nickname, room_sender))
}

async fn handle_socket(
    socket: WebSocket,
    nickname: String,
    room_sender: broadcast::Sender<(Uuid, String)>,
) {
    let mut receiver = room_sender.subscribe();
    let (mut write, mut read) = socket.split();
    let my_id = Uuid::new_v4();

    tokio::spawn(async move {
        while let Ok((sender_id, msg)) = receiver.recv().await {
            if sender_id != my_id {
                if write.send(Message::Text(msg.clone().into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let _ = room_sender.send((my_id, format!("[{} joined the room!]", nickname)));

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Text(text) = msg {
            let message = format!("[{}] {}", nickname, text);
            let _ = room_sender.send((my_id, message));
        }
    }
}

#[tokio::main]
async fn main() {
    let state: StateType = Arc::new(RwLock::new(HashMap::new()));
    let app = Router::new()
        .route("/login/{room}/{nickname}", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}