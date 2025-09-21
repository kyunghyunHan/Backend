use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://echo.websocket.events";
    println!("🔌 서버 연결 중: {}", url);

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    println!("✅ 연결 성공!");

    let msg = "Hello WebSocket!";
    write.send(Message::Text(msg.into())).await?;
    println!("📤 보냄: {}", msg);

    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => println!("📥 수신: {}", text),
            Message::Close(_) => break,
            _ => {}
        }
    }
    Ok(())
}
