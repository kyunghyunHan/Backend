use tonic::{transport::Server, Request, Response, Status};
use tonic::transport::Channel;

// Proto 정의
pub mod calculator {
    tonic::include_proto!("calculator");
}

use calculator::calculator_server::{Calculator, CalculatorServer};
use calculator::calculator_client::CalculatorClient;
use calculator::{AddRequest, AddResponse};

#[derive(Debug, Default)]
pub struct CalculatorService {}

#[tonic::async_trait]
impl Calculator for CalculatorService {
    async fn add(&self, request: Request<AddRequest>) -> Result<Response<AddResponse>, Status> {
        let req = request.into_inner();
        let result = req.a + req.b;
        
        println!("[gRPC Server] 계산: {} + {} = {}", req.a, req.b, result);
        
        let response = AddResponse { result };
        Ok(Response::new(response))
    }
}

// gRPC 서버
async fn grpc_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:50051".parse()?;
    let calculator = CalculatorService::default();

    println!("[gRPC Server] {}에서 대기 중...", addr);

    Server::builder()
        .add_service(CalculatorServer::new(calculator))
        .serve(addr)
        .await?;

    Ok(())
}

// gRPC 클라이언트
async fn grpc_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[gRPC Client] 클라이언트 시작");
    
    let mut client = CalculatorClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(AddRequest { a: 10, b: 20 });
    println!("[gRPC Client] 요청 전송: 10 + 20");
    
    let response = client.add(request).await?;
    
    println!("[gRPC Client] 서버에서 받은 결과: {}", response.into_inner().result);

    Ok(())
}

#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(">> gRPC 예제 시작");

    let grpc_server_handle = tokio::spawn(async {
        if let Err(e) = grpc_server().await {
            println!("[gRPC Server] 에러: {}", e);
        }
    });

    // 서버 시작 대기
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    if let Err(e) = grpc_client().await {
        println!("[gRPC Client] 에러: {}", e);
    }

    // 서버 종료
    grpc_server_handle.abort();

    println!("\n완료!");

    Ok(())
}