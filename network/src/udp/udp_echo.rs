use tokio::net::UdpSocket;

// ==================== UDP ECHO SERVER ====================

async fn udp_echo_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:8081").await?;
    println!("[UDP SERVER] Echo server listening on 127.0.0.1:8081");

    let mut buffer = [0u8; 1024];

    loop {
        let (n, addr) = socket.recv_from(&mut buffer).await?;

        let received = &buffer[..n];

        println!(
            "[UDP SERVER] Received from {}: {}",
            addr,
            String::from_utf8_lossy(received)
        );

        socket.send_to(received, addr).await?;
    }
}

// ==================== UDP CLIENT ====================

async fn udp_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    println!("[UDP CLIENT] Bound to local random port");

    let server_addr = "127.0.0.1:8081";
    let message = "Hello UDP Echo Server!";

    socket.send_to(message.as_bytes(), server_addr).await?;
    println!("[UDP CLIENT] Sent: {}", message);

    let mut buffer = [0u8; 1024];
    let (n, addr) = socket.recv_from(&mut buffer).await?;

    let response = String::from_utf8_lossy(&buffer[..n]);
    println!("[UDP CLIENT] Received from {}: {}", addr, response);

    Ok(())
}

// ==================== MAIN ====================

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting UDP Echo Example\n");

    tokio::spawn(async {
        udp_echo_server().await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    udp_client().await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    Ok(())
}