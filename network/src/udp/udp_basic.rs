use tokio::net::{TcpListener, TcpStream, UdpSocket};

// UDP 서버
async fn udp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:8081").await?;
    println!("[UDP Server] 127.0.0.1:8081에서 대기 중...");

    let mut buffer = [0; 1024];
    let (size, addr) = socket.recv_from(&mut buffer).await?;
    let received = String::from_utf8_lossy(&buffer[..size]);
    println!("[UDP Server] {}에서 받은 메시지: {}", addr, received);

    let response = "Hello from UDP Server!";
    socket.send_to(response.as_bytes(), addr).await?;
    println!("[UDP Server] 응답 전송: {}", response);

    Ok(())
}

// UDP 클라이언트
async fn udp_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    println!("[UDP Client] 클라이언트 시작");

    let message = "Hello from UDP Client!";
    socket.send_to(message.as_bytes(), "127.0.0.1:8081").await?;
    println!("[UDP Client] 메시지 전송: {}", message);

    let mut buffer = [0; 1024];
    let (size, addr) = socket.recv_from(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..size]);
    println!("[UDP Client] {}에서 받은 응답: {}", addr, response);

    Ok(())
}

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(">> UDP 예제 시작");

    let udp_server_handle = tokio::spawn(async {
        if let Err(e) = udp_server().await {
            println!("[UDP Server] 에러: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    if let Err(e) = udp_client().await {
        println!("[UDP] 에러: {}", e);
    }

    let _ = udp_server_handle.await;

    println!("\n완료!");

    Ok(())
}
