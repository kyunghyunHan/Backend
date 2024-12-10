use futures_lite::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,
};
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<()> {
    // 로깅 설정
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();

    // RabbitMQ 연결 주소
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    // RabbitMQ 연결
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    ).await?;
    
    info!("RabbitMQ에 연결됨");

    // 채널 생성
    let channel = conn.create_channel().await?;

    // 큐 선언
    let queue = channel
        .queue_declare(
            "hello",                              // 큐 이름
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!(?queue, "큐에 연결됨");

    // Consumer 설정
    let mut consumer = channel
        .basic_consume(
            "hello",                              // 큐 이름
            "my_consumer",                        // consumer 태그
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("메시지 수신 대기 시작...");

    // 메시지 수신 루프
    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                // 메시지 내용을 문자열로 변환
                match std::str::from_utf8(&delivery.data) {
                    Ok(message) => {
                        info!("수신한 메시지: {}", message);
                        
                        // 여기에 메시지 처리 로직 추가
                        
                        // 메시지 처리 완료 확인
                        delivery
                            .ack(BasicAckOptions::default())
                            .await
                            .expect("메시지 확인 실패");
                    },
                    Err(e) => {
                        info!("잘못된 메시지 형식: {}", e);
                    }
                }
            },
            Err(e) => {
                info!("메시지 수신 오류: {}", e);
            }
        }
    }

    Ok(())
}