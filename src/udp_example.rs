use crate::connection::connection;
use tokio::net::UdpSocket;

#[tokio::main]
pub async fn example() {
    let connection = connection().await;

    // UDP 소켓을 바인딩 (127.0.0.1의 8080 포트)
    let socket = UdpSocket::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on 127.0.0.1:8080");

    let mut buf = vec![0u8; 1024]; // 1024 바이트 버퍼

    loop {
        // 클라이언트로부터 데이터 수신
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap();

        println!("Received {} bytes from {}", len, addr);
        if let Ok(received_data) = std::str::from_utf8(&buf[..len]) {
            println!("Received data: {}", received_data);
        } else {
            println!("Received non-UTF8 data: {:?}", &buf[..len]);
        }
        let result =
            sqlx::query("INSERT INTO game_test (aa) VALUES ($1)")
                .bind(std::str::from_utf8(&buf[..len]).unwrap()) // user_id 값 바인딩
                .execute(&connection) // 쿼리 실행
                .await
                .expect("Failed to insert user");

        println!("{:?}", result);
        // 수신한 데이터를 그대로 클라이언트에게 전송 (에코)
        socket.send_to(&buf[..len], &addr).await.unwrap();

        println!("Echoed {} bytes to {}", len, addr);
    }
}
