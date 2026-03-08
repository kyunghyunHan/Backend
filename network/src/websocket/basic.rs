use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // TLS를 위한 암호화 공급자 초기화
    CryptoProvider::install_default(default_provider()).unwrap();

    // Binance BTC/USDT 실시간 거래 스트림에 연결
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    println!("🔌 {}에 연결 중...", url);

    // WebSocket 연결 설정
    let (ws_stream, _response) = connect_async(url).await?;
    println!("✅ 성공적으로 연결되었습니다!");

    // 스트림을 송신자와 수신자로 분할
    let (_write, mut read) = ws_stream.split();

    // 메시지 처리 루프
    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                println!("📥 수신됨: {}", text);
            }
            Message::Ping(payload) => {
                println!("🏓 핑 수신: {:?}", payload);
            }
            Message::Close(_) => {
                println!("🔌 서버가 연결을 종료했습니다");
                break;
            }
            _ => {
                // 기타 메시지 타입 처리 (Binary, Pong 등)
            }
        }
    }

    Ok(())
}