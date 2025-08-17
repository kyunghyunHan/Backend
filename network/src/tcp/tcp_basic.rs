use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
// ==================== TCP 함수들 ====================

// TCP 서버
async fn tcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[TCP Server] 127.0.0.1:8080에서 대기 중...");

    let (mut socket, addr) = listener.accept().await?;
    println!("[TCP Server] 클라이언트 연결됨: {}", addr);

    let mut buffer = [0; 1024];
    let n = socket.read(&mut buffer).await?;
    let received = String::from_utf8_lossy(&buffer[..n]);
    println!("[TCP Server] 받은 메시지: {}", received);

    let response = "Hello from TCP Server!";
    socket.write_all(response.as_bytes()).await?;
    println!("[TCP Server] 응답 전송: {}", response);

    Ok(())
}

// TCP 클라이언트
async fn tcp_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("[TCP Client] 서버에 연결됨");

    let message = "Hello from TCP Client!";
    stream.write_all(message.as_bytes()).await?;
    println!("[TCP Client] 메시지 전송: {}", message);

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    println!("[TCP Client] 받은 응답: {}", response);

    Ok(())
}



#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== TCP/UDP Hello World 예제 ===\n");
    
    // TCP 예제
    println!(">> TCP 예제 시작");
    let tcp_server_handle = tokio::spawn(tcp_server());
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    tcp_client().await?;
    tcp_server_handle.await??;
    
    println!("\n{}\n", "=".repeat(30));
    
    // UDP 예제
   
    Ok(())
}
