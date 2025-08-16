use network::tcp::tcp_basic::{tcp_client, tcp_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== TCP/UDP Hello World 예제 ===\n");

    // TCP 예제 실행
    println!(">> TCP 예제 시작");
    
    // TCP 서버를 별도 태스크에서 실행
    let tcp_server_handle = tokio::spawn(async {
        if let Err(e) = tcp_server().await {
            println!("[TCP Server] 에러: {}", e);
        }
    });
    
    // 서버가 시작될 시간을 줌
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // TCP 클라이언트 실행
    if let Err(e) = tcp_client().await {
        println!("[TCP] 에러: {}", e);
    }
    
    // TCP 서버 종료 대기
    let _ = tcp_server_handle.await;
    
    // 문자열 연결 수정
    println!("\n{}\n", "=".repeat(30));

    println!("\n완료!");
    Ok(())
}