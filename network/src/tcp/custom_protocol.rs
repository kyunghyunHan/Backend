use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const HEADER_SIZE: usize = 4;

// ==================== PROTOCOL HELPERS ====================

async fn write_frame(
    stream: &mut TcpStream,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let payload = message.as_bytes();
    let len = payload.len() as u32;

    // 4바이트 big-endian length header
    let header = len.to_be_bytes();

    stream.write_all(&header).await?;
    stream.write_all(payload).await?;

    Ok(())
}

async fn read_frame(
    stream: &mut TcpStream,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut header = [0u8; HEADER_SIZE];

    // 먼저 길이 4바이트 읽기
    match stream.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            // 연결 종료
            return Ok(None);
        }
        Err(e) => return Err(Box::new(e)),
    }

    let len = u32::from_be_bytes(header) as usize;

    if len == 0 {
        return Ok(Some(String::new()));
    }

    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;

    let message = String::from_utf8(payload)?;
    Ok(Some(message))
}

// ==================== SERVER ====================

async fn handle_client(
    mut socket: TcpStream,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[SERVER] Client connected: {}", addr);

    loop {
        match read_frame(&mut socket).await? {
            Some(message) => {
                println!("[SERVER] Received from {}: {}", addr, message);

                let response = format!("echo: {}", message);
                write_frame(&mut socket, &response).await?;
            }
            None => {
                println!("[SERVER] Client disconnected: {}", addr);
                return Ok(());
            }
        }
    }
}

async fn custom_protocol_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[SERVER] Length-prefixed server listening on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, addr).await {
                println!("[SERVER] Error handling {}: {}", addr, e);
            }
        });
    }
}

// ==================== CLIENT ====================

async fn custom_protocol_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("[CLIENT] Connected to server");

    let messages = vec![
        "hello",
        "this is custom protocol",
        "rust tokio framing test",
    ];

    for msg in messages {
        println!("[CLIENT] Sending: {}", msg);

        write_frame(&mut stream, msg).await?;

        match read_frame(&mut stream).await? {
            Some(response) => {
                println!("[CLIENT] Received: {}", response);
            }
            None => {
                println!("[CLIENT] Server closed connection");
                break;
            }
        }
    }

    Ok(())
}

// ==================== MAIN ====================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting Length-Prefixed Custom Protocol Example\n");

    tokio::spawn(async {
        if let Err(e) = custom_protocol_server().await {
            println!("[SERVER] Fatal error: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    custom_protocol_client().await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    Ok(())
}