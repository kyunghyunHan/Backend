// WebSocket = 실시간 양방향 통신
// HTTP는 요청->응답 끝
// WebSocket은 계속 연결되어서 실시간으로 데이터 주고받기

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

// 1. 기본 WebSocket 클라이언트
#[tokio::main]
async fn basic_websocket() -> Result<(), Box<dyn std::error::Error>> {
    // 바이낸스 실시간 가격 스트림에 연결
    let url = "wss://stream.binance.com:9443/ws/btcusdt@ticker";
    
    println!("📡 바이낸스에 연결 중...");
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    println!("✅ 연결됨! 실시간 BTC 가격 수신 중...");
    
    // 메시지 받기
    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                // JSON 파싱해서 가격만 출력
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(price) = json["c"].as_str() {
                        println!("💰 BTC 가격: ${}", price);
                    }
                }
            }
            Message::Close(_) => {
                println!("🔌 연결 종료됨");
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

// 2. 여러 코인 동시 모니터링
#[tokio::main]
async fn multi_coin_monitor() -> Result<(), Box<dyn std::error::Error>> {
    let coins = vec!["btcusdt", "ethusdt", "adausdt"];
    let mut tasks = Vec::new();
    
    for coin in coins {
        let url = format!("wss://stream.binance.com:9443/ws/{}@ticker", coin);
        
        let task = tokio::spawn(async move {
            if let Ok((ws_stream, _)) = connect_async(&url).await {
                let (_, mut read) = ws_stream.split();
                
                while let Some(Ok(Message::Text(text))) = read.next().await {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let (Some(symbol), Some(price)) = 
                            (json["s"].as_str(), json["c"].as_str()) {
                            println!("📈 {}: ${}", symbol, price);
                        }
                    }
                }
            }
        });
        
        tasks.push(task);
    }
    
    // 모든 스트림이 끝날 때까지 대기
    for task in tasks {
        let _ = task.await;
    }
    
    Ok(())
}

// 3. HFT용 - 주문서(Order Book) 실시간 수신
#[tokio::main]
async fn orderbook_stream() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://stream.binance.com:9443/ws/btcusdt@depth20@100ms";
    
    let (ws_stream, _) = connect_async(url).await?;
    let (_, mut read) = ws_stream.split();
    
    println!("📊 실시간 주문서 수신 중...");
    
    while let Some(Ok(Message::Text(text))) = read.next().await {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            // 매수/매도 호가 출력
            if let (Some(bids), Some(asks)) = 
                (json["bids"].as_array(), json["asks"].as_array()) {
                
                println!("\n=== BTC 주문서 ===");
                
                // 최고 매수가 (bid)
                if let Some(best_bid) = bids.first() {
                    if let (Some(price), Some(qty)) = 
                        (best_bid[0].as_str(), best_bid[1].as_str()) {
                        println!("🟢 최고 매수: ${} (수량: {})", price, qty);
                    }
                }
                
                // 최저 매도가 (ask)  
                if let Some(best_ask) = asks.first() {
                    if let (Some(price), Some(qty)) = 
                        (best_ask[0].as_str(), best_ask[1].as_str()) {
                        println!("🔴 최저 매도: ${} (수량: {})", price, qty);
                    }
                }
                
                // 스프레드 계산
                if let (Some(bid), Some(ask)) = (bids.first(), asks.first()) {
                    if let (Ok(bid_price), Ok(ask_price)) = 
                        (bid[0].as_str().unwrap().parse::<f64>(), 
                         ask[0].as_str().unwrap().parse::<f64>()) {
                        let spread = ask_price - bid_price;
                        println!("📏 스프레드: ${:.2}", spread);
                    }
                }
            }
        }
    }
    
    Ok(())
}

// 4. 메시지 보내기 (주문 등)
#[tokio::main]
async fn send_messages() -> Result<(), Box<dyn std::error::Error>> {
    // 실제로는 인증된 프라이빗 스트림 사용
    let url = "ws://localhost:8080/trading"; // 예시 URL
    
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // 주문 메시지 보내기
    let order_message = serde_json::json!({
        "type": "new_order",
        "symbol": "BTCUSDT", 
        "side": "BUY",
        "quantity": 0.001,
        "price": 50000.0
    });
    
    write.send(Message::Text(order_message.to_string().into())).await?;
    println!("📤 주문 전송됨");
    
    // 응답 받기
    if let Some(Ok(Message::Text(response))) = read.next().await {
        println!("📥 응답: {}", response);
    }
    
    Ok(())
}

// 의존성 추가 필요:
/*
[dependencies]
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.20"
futures-util = "0.3"
serde_json = "1.0"
*/