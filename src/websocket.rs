use futures_util::{SinkExt, StreamExt};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio::sync::Mutex;
use crate::connection::connection;

// 클라이언트를 관리할 구조체
type Client = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;
type Rooms = Arc<Mutex<HashMap<String, Vec<Client>>>>;

async fn handle_connection(stream: tokio::net::TcpStream, rooms: Rooms) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the WebSocket handshake");
    println!("New WebSocket connection");
    let connection = connection().await;

    let (mut write, mut read) = ws_stream.split();

    // 클라이언트가 메시지를 보낼 때 방 선택 및 메시지 전송
    while let Some(Ok(message)) = read.next().await {
        if let Ok(text) = message.to_text() {
            let parts: Vec<&str> = text.splitn(2, ":").collect();
            if parts.len() != 2 {
                println!("Invalid message format. Expected 'room:message'");
                continue;
            }
            let room_name = parts[0]; // 방 이름
            let chat_message = parts[1]; // 메시지

            println!("Room: {}, Message: {}", room_name, chat_message);

            // 데이터베이스에 메시지 저장
            sqlx::query("INSERT INTO game_test (aa) VALUES ($1)")
                .bind(chat_message)
                .execute(&connection)
                .await
                .expect("Failed to insert data");

            // 해당 방에 속한 클라이언트들에게 메시지 전송
            let mut rooms_guard = rooms.lock().await;
            if let Some(mut clients) = rooms_guard.get_mut(room_name) {
                for client in clients.iter_mut() {
                    client.send(message.clone()).await.unwrap();
                }
            } else {
                println!("Room not found: {}", room_name);
            }
        }
    }
}

#[tokio::main]
pub async fn example() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new())); // 방 데이터 관리

    println!("Listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        let rooms = rooms.clone();
        let connection = connection.clone();
        tokio::spawn(handle_connection(stream, rooms));
    }
}
