use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // 테스트용 공개 에코 서버 (보낸 메시지를 그대로 돌려줌)
    let url = "wss://echo.websocket.events";
    println!("🔌 서버 연결 중: {}", url);

    // WebSocket 연결
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    println!("✅ 연결 성공!");

    // 메시지 전송
    let msg = "Hello WebSocket!";
    write.send(Message::Text(msg.to_string().into())).await?;
    println!("📤 보냄: {}", msg);

    // 응답 수신
    if let Some(Ok(Message::Text(text))) = read.next().await {
        println!("📥 수신: {}", text);
    }

    Ok(())
}