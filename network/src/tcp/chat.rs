use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

async fn handle_client(
    socket: TcpStream,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<String>,
) {
    println!("[SERVER] Client connected: {}", addr);

    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // 이 클라이언트도 전체 채널을 구독
    let mut rx = tx.subscribe();

    // 입장 메시지
    let join_msg = format!("[{}] joined the chat", addr);
    let _ = tx.send(join_msg);

    loop {
        tokio::select! {
            // 1) 클라이언트가 보낸 메시지 읽기
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        println!("[SERVER] Client disconnected: {}", addr);
                        let leave_msg = format!("[{}] left the chat", addr);
                        let _ = tx.send(leave_msg);
                        return;
                    }
                    Ok(_) => {
                        let msg = line.trim().to_string();

                        if !msg.is_empty() {
                            let full_msg = format!("[{}] {}", addr, msg);
                            let _ = tx.send(full_msg);
                        }

                        line.clear();
                    }
                    Err(e) => {
                        println!("[SERVER] Read error from {}: {}", addr, e);
                        let leave_msg = format!("[{}] left due to error", addr);
                        let _ = tx.send(leave_msg);
                        return;
                    }
                }
            }

            // 2) 다른 사람이 보낸 메시지를 이 클라이언트에게 전달
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            println!("[SERVER] Write error to {}: {}", addr, e);
                            return;
                        }
                        if let Err(e) = writer.write_all(b"\n").await {
                            println!("[SERVER] Write error to {}: {}", addr, e);
                            return;
                        }
                    }
                    Err(e) => {
                        println!("[SERVER] Broadcast receive error for {}: {}", addr, e);
                        return;
                    }
                }
            }
        }
    }
}

async fn tcp_chat_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[SERVER] TCP Chat Server listening on 127.0.0.1:8080");

    // broadcast 채널 생성
    let (tx, _) = broadcast::channel::<String>(100);

    loop {
        let (socket, addr) = listener.accept().await?;
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            handle_client(socket, addr, tx_clone).await;
        });
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tcp_chat_server().await
}