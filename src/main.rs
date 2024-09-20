use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{stream::StreamExt, SinkExt}; // SinkExt와 StreamExt 모두 import
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WSMessage};
type Tx = broadcast::Sender<String>;
type Rx = broadcast::Receiver<String>;


pub async fn index() -> Html<&'static str> {
    let html_content = include_str!("../index.html");
    Html(html_content)
}
#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel::<String>(100);

    // 경로 설정: `/`에서 index.html 제공, `/ws`에서 WebSocket 처리
    let app = Router::new()
        .route("/", get(index)) // index.html 제공
        .route("/ws", get(move |ws: WebSocketUpgrade| handle_ws(ws, tx.clone())));
                                       // 나머지 서버 설정
                                       // let addr = listener.local_addr().unwrap();
                                       // println!("listening on {}", listener.local_addr().unwrap());
                                       // axum::serve(listener, app).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
    // tokio::spawn(async move {
    //     if let Ok((mut socket, _)) = connect_async("ws://127.0.0.1:3001/ws").await {
    //         println!("Connected to 3000");

    //         // 메시지 보내기
    //         socket
    //             .send(WSMessage::Text("Hello from 30001".into()))
    //             .await
    //             .unwrap();

    //         // 메시지 수신
    //         while let Some(msg) = socket.next().await {
    //             if let Ok(WSMessage::Text(text)) = msg {
    //                 println!("Received from 3000: {}", text);
    //             }
    //         }
    //     }
    // })
    // .await
    // .unwrap();
}
async fn handle_ws(ws: WebSocketUpgrade, tx: Tx) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}
async fn handle_socket(mut socket: WebSocket, tx: Tx) {
    let mut rx = tx.subscribe(); // 브로드캐스트 수신자 구독
    let (mut sender, mut receiver) = socket.split();

    // 클라이언트로부터 받은 메시지를 브로드캐스트
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            println!("Received: {}", text);
            // 모든 클라이언트에게 메시지 브로드캐스트
            let _ = tx_clone.send(text);
        }
    });

    // 다른 클라이언트의 메시지를 수신해서 현재 클라이언트에게 전달
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
}