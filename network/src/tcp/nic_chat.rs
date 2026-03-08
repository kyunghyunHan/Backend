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

    // 닉네임 입력 요청
    if let Err(e) = writer.write_all(b"Enter your nickname: ").await {
        println!("[SERVER] Failed to ask nickname {}: {}", addr, e);
        return;
    }

    let mut nickname = String::new();

    match reader.read_line(&mut nickname).await {
        Ok(0) => {
            println!("[SERVER] Client disconnected before nickname: {}", addr);
            return;
        }
        Ok(_) => {}
        Err(e) => {
            println!("[SERVER] Failed to read nickname from {}: {}", addr, e);
            return;
        }
    }

    let nickname = nickname.trim().to_string();

    if nickname.is_empty() {
        let _ = writer.write_all(b"Nickname cannot be empty. Bye.\n").await;
        println!("[SERVER] Empty nickname from {}", addr);
        return;
    }

    println!("[SERVER] {} set nickname: {}", addr, nickname);

    let welcome_msg = format!("*** Welcome, {}!\n", nickname);
    if let Err(e) = writer.write_all(welcome_msg.as_bytes()).await {
        println!("[SERVER] Failed to send welcome message to {}: {}", addr, e);
        return;
    }

    let join_msg = format!("*** {} joined the chat", nickname);
    let _ = tx.send(join_msg);

    // 이 클라이언트도 전체 채널 구독
    let mut rx = tx.subscribe();

    let mut line = String::new();

    loop {
        tokio::select! {
            // 클라이언트 입력 처리
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        println!("[SERVER] Client disconnected: {} ({})", addr, nickname);
                        let leave_msg = format!("*** {} left the chat", nickname);
                        let _ = tx.send(leave_msg);
                        return;
                    }
                    Ok(_) => {
                        let msg = line.trim().to_string();

                        if !msg.is_empty() {
                            if msg == "/quit" {
                                let bye = b"*** Bye!\n";
                                let _ = writer.write_all(bye).await;

                                let leave_msg = format!("*** {} left the chat", nickname);
                                let _ = tx.send(leave_msg);

                                println!("[SERVER] Client quit: {} ({})", addr, nickname);
                                return;
                            }

                            let full_msg = format!("[{}] {}", nickname, msg);
                            let _ = tx.send(full_msg);
                        }

                        line.clear();
                    }
                    Err(e) => {
                        println!("[SERVER] Read error from {} ({}): {}", addr, nickname, e);
                        let leave_msg = format!("*** {} left due to error", nickname);
                        let _ = tx.send(leave_msg);
                        return;
                    }
                }
            }

            // 브로드캐스트 메시지 전달
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            println!("[SERVER] Write error to {} ({}): {}", addr, nickname, e);
                            return;
                        }

                        if let Err(e) = writer.write_all(b"\n").await {
                            println!("[SERVER] Write error to {} ({}): {}", addr, nickname, e);
                            return;
                        }
                    }
                    Err(e) => {
                        println!("[SERVER] Broadcast receive error for {} ({}): {}", addr, nickname, e);
                        return;
                    }
                }
            }
        }
    }
}

async fn tcp_chat_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("[SERVER] Nickname Chat Server listening on 127.0.0.1:8080");

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