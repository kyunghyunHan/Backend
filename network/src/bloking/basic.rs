// 1. 블로킹 방식 (나쁜 예)
fn blocking_way() {
    println!("1번 작업 시작");
    std::thread::sleep(std::time::Duration::from_secs(2)); // 2초 기다림
    println!("1번 작업 끝");
    
    println!("2번 작업 시작");
    std::thread::sleep(std::time::Duration::from_secs(2)); // 또 2초 기다림
    println!("2번 작업 끝");
    
    // 총 4초 걸림 😭
}

// 2. 논블로킹 방식 (좋은 예)
#[tokio::main]
pub async fn nonblocking_way() {
    let task1 = tokio::spawn(async {
        println!("1번 작업 시작");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("1번 작업 끝");
    });
    
    let task2 = tokio::spawn(async {
        println!("2번 작업 시작");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("2번 작업 끝");
    });
    
    // 두 작업을 동시에 실행
    let _ = tokio::join!(task1, task2);
    
    // 총 2초만 걸림! 🎉
}

// 3. HFT 예제 - 여러 거래소 가격 동시 확인
#[tokio::main]
pub async fn check_prices() {
    let binance = tokio::spawn(async {
        // 바이낸스 API 호출 (시뮬레이션)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("바이낸스: BTC = $50,000");
        50000.0
    });
    
    let coinbase = tokio::spawn(async {
        // 코인베이스 API 호출 (시뮬레이션)
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        println!("코인베이스: BTC = $50,100");
        50100.0
    });
    
    let kraken = tokio::spawn(async {
        // 크라켄 API 호출 (시뮬레이션)
        tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
        println!("크라켄: BTC = $49,950");
        49950.0
    });
    
    // 모든 가격을 동시에 가져옴 (가장 느린 것만큼만 기다림)
    let (binance_price, coinbase_price, kraken_price) = 
        tokio::join!(binance, coinbase, kraken);
    
    let binance_price = binance_price.unwrap();
    let coinbase_price = coinbase_price.unwrap();
    let kraken_price = kraken_price.unwrap();
    
    // 차익거래 기회 찾기
    if coinbase_price > binance_price + 50.0 {
        println!("🚀 차익거래 기회! 바이낸스에서 사고 코인베이스에서 팔기");
    }
}