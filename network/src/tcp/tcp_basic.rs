use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
pub async fn tcp_server() -> Result<(), Box<dyn std::error::Error>> {
    // 127.0.0.1:8080에서 대기
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[TCP Server] 127.0.0.1:8080에서 대기 중...");

    // 클라이언트 연결 대기
    let (mut socket, addr) = listener.accept().await?;
    println!("[TCP Server] 클라이언트 연결됨: {}", addr);

    // 메시지 읽기
    let mut buffer = [0; 1024];
    let n = socket.read(&mut buffer).await?;
    let received = String::from_utf8_lossy(&buffer[..n]);
    println!("[TCP Server] 받은 메시지: {}", received);

    // 응답 보내기
    let response = "Hello from TCP Server!";
    socket.write_all(response.as_bytes()).await?;
    println!("[TCP Server] 응답 전송: {}", response);

    Ok(())
}

pub async fn tcp_client() -> Result<(), Box<dyn std::error::Error>> {
    // 서버에 연결
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("[TCP Client] 서버에 연결됨");

    // 메시지 보내기
    let message = "Hello from TCP Client!";
    stream.write_all(message.as_bytes()).await?;
    println!("[TCP Client] 메시지 전송: {}", message);

    // 응답 읽기
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    println!("[TCP Client] 받은 응답: {}", response);

    Ok(())
}
