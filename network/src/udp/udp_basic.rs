use tokio::net::UdpSocket;

// ==================== UDP Functions ====================

// UDP Server
async fn udp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:8081").await?;
    println!("[UDP Server] Listening on 127.0.0.1:8081...");

    let mut buffer = [0; 1024];
    let (size, addr) = socket.recv_from(&mut buffer).await?;
    let received = String::from_utf8_lossy(&buffer[..size]);
    println!("[UDP Server] Received message from {}: {}", addr, received);

    let response = "Hello from UDP Server!";
    socket.send_to(response.as_bytes(), addr).await?;
    println!("[UDP Server] Response sent: {}", response);

    Ok(())
}

// UDP Client
async fn udp_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    println!("[UDP Client] Client started");

    let message = "Hello from UDP Client!";
    socket.send_to(message.as_bytes(), "127.0.0.1:8081").await?;
    println!("[UDP Client] Message sent: {}", message);

    let mut buffer = [0; 1024];
    let (size, addr) = socket.recv_from(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..size]);
    println!("[UDP Client] Received response from {}: {}", addr, response);

    Ok(())
}

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== UDP Hello World Example ===\n");
    
    // UDP Example
    println!(">> UDP Example Started");

    let udp_server_handle = tokio::spawn(async {
        if let Err(e) = udp_server().await {
            println!("[UDP Server] Error: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    if let Err(e) = udp_client().await {
        println!("[UDP] Error: {}", e);
    }

    let _ = udp_server_handle.await;

    println!("\nCompleted!");

    Ok(())
}