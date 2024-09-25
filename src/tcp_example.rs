use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Notify};
use tokio::task;
use std::sync::atomic::{AtomicBool, Ordering};

struct JdListener {
    accept_notify: Arc<Notify>,
    is_thread_live: Arc<AtomicBool>,
    tx: broadcast::Sender<String>, 
}

impl JdListener {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self {
            accept_notify: Arc::new(Notify::new()),
            is_thread_live: Arc::new(AtomicBool::new(true)),
            tx,
        }
    }

    pub async fn start(&self, host: &str, port: u16) {
        let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&addr).await.unwrap();

        let is_thread_live = self.is_thread_live.clone();
        let accept_notify = self.accept_notify.clone();
        let tx = self.tx.clone();

        task::spawn(async move {
            while is_thread_live.load(Ordering::Relaxed) {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        println!("New client connected");
                        let socket = Arc::new(Mutex::new(socket)); // Arc<Mutex<TcpStream>>으로 감쌈
                        Self::handle_client(socket, tx.clone()).await;
                    }
                    Err(e) => {
                        println!("Failed to accept connection: {}", e);
                    }
                }

                // 클라이언트 접속 처리 후 다시 대기 상태로 돌아감
                accept_notify.notify_one();
            }
        });
    }

    async fn handle_client(socket: Arc<Mutex<TcpStream>>, tx: broadcast::Sender<String>) {
        let client_addr = socket.lock().unwrap().peer_addr().unwrap();
        println!("Handling client: {:?}", client_addr);

        // 클라이언트가 접속했을 때 처리할 로직을 여기에 추가
        // 예: 메시지 수신, 송신 로직 등

        // 브로드캐스트 메시지를 보내는 예시:
        let msg = format!("Client connected: {:?}", client_addr);
        let _ = tx.send(msg);
    }

    // 서버를 종료하는 함수
    pub fn close(&self) {
        self.is_thread_live.store(false, Ordering::Relaxed);
    }
}

#[tokio::main]
pub async  fn example() {
    let (tx, _rx) = broadcast::channel(100);

    // jdListener 생성
    let listener = JdListener::new(tx.clone());

    // TCP 리스너 시작
    listener.start("127.0.0.1", 4000).await;

    println!("TCP server running on 127.0.0.1:4000");

    // 서버가 계속 실행되도록 대기
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
