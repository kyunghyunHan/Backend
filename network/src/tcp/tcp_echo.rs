
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::net::{TcpListener,TcpStream};

async fn handle_client(mut soket:TcpStream,addr:std::net::SocketAddr){
    println!("[SERVER] Client connected: {}", addr);
    
    let mut buffer = [0u8;1024];
    loop{
        match socket.read(&mut buffer).await{
         Ok(0)=>{
            println!("[SERVER] Client disconnected: {}", addr);
            return;
         }
         Ok(n)=>{
            let received = &buffer[..n];
            println!(
                "[SERVER] Received from {}: {}",
                addr,
                String::from_utf8_lossy(received)
            );
            if let Err(e) = socket.write_all(received).await {
                println!("[SERVER] Write error: {}", e);
                return;
            }
        }
        Err(e)=>{
            println!("[SERVER] Read error: {}", e);
            return;        }
        }
    }

}

async fn tcp_echo_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("[SERVER] Echo server listening on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;

        // 클라이언트마다 새로운 task 생성
        tokio::spawn(handle_client(socket, addr));
    }
}


async fn tcp_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    println!("[CLIENT] Connected to server");

    let message = "Hello Echo Server!";
    stream.write_all(message.as_bytes()).await?;

    println!("[CLIENT] Sent: {}", message);

    let mut buffer = [0u8; 1024];

    let n = stream.read(&mut buffer).await?;

    let response = String::from_utf8_lossy(&buffer[..n]);

    println!("[CLIENT] Received Echo: {}", response);

    Ok(())
}


#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting TCP Echo Example\n");

    // 서버 실행
    let server = tokio::spawn(async {
        tcp_echo_server().await.unwrap();
    });

    // 서버 시작 대기
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 클라이언트 실행
    tcp_client().await?;

    // 서버 계속 실행
    server.await?;

    Ok(())
}