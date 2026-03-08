use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut socket: TcpStream, addr: std::net::SocketAddr) {
    println!("[SERVER] Client connected: {}", addr);

    let mut buffer = [0u8; 1024];

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("[SERVER] Client disconnected: {}", addr);
                return;
            }
            Ok(n) => {
                let received = &buffer[..n];

                println!(
                    "[SERVER] Received from {}: {}",
                    addr,
                    String::from_utf8_lossy(received)
                );

                if let Err(e) = socket.write_all(received).await {
                    println!("[SERVER] Write error for {}: {}", addr, e);
                    return;
                }
            }
            Err(e) => {
                println!("[SERVER] Read error for {}: {}", addr, e);
                return;
            }
        }
    }
}

async fn tcp_multi_client_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[SERVER] Multi-client TCP server listening on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(handle_client(socket, addr));
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tcp_multi_client_server().await
}