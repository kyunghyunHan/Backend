use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::Html,
    routing::{get, post},
    Router,
};

#[derive(SimpleObject)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub message: String,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        println!("[GraphQL Server] Hello 요청 받음");
        "Hello, GraphQL World!".to_string()
    }

    async fn user(&self, id: i32) -> User {
        println!("[GraphQL Server] 유저 {} 조회", id);
        User {
            id,
            name: format!("tuser{}", id),
            message: "안녕하세요".to_string(),
        }
    }
    async fn add(&self, a: i32, b: i32) -> i32 {
        let result = a + b;
        println!("[GraphQL Server] 계산: {} + {} = {}", a, b, result);
        result
    }
}

//Schema
pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;


async fn graphql_handler(
    schema: Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}


// GraphQL Playground
async fn graphql_playground() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>GraphQL Playground</title>
        </head>
        <body>
            <div id="root">
                <h1>GraphQL Hello World</h1>
                <p>POST /graphql 로 요청하세요!</p>
                <h2>예시 쿼리들:</h2>
                <pre>
query {
  hello
}

query {
  user(id: 1) {
    id
    name
    message
  }
}

query {
  add(a: 10, b: 20)
}
                </pre>
            </div>
        </body>
        </html>
        "#,
    )
}

async fn graphql_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[GraphQL Server] 127.0.0.1:3000에서 대기 중...");

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// GraphQL 클라이언트 (HTTP로 요청)
async fn graphql_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[GraphQL Client] 클라이언트 시작");

    let client = reqwest::Client::new();

    let query1 = r#"{"query": "{ hello }"}"#;
    println!("[GraphQL Client] Hello 요청 전송");
    
    let response1 = client
        .post("http://127.0.0.1:3000/graphql")
        .header("Content-Type", "application/json")
        .body(query1)
        .send()
        .await?;
    
    let result1: serde_json::Value = response1.json().await?;
    println!("[GraphQL Client] 응답: {}", result1["data"]["hello"].as_str().unwrap());


    let query2 = r#"{"query": "{ user(id: 42) { id name message } }"}"#;
    println!("[GraphQL Client] 사용자 요청 전송");
    
    let response2 = client
        .post("http://127.0.0.1:3000/graphql")
        .header("Content-Type", "application/json")
        .body(query2)
        .send()
        .await?;
    
    let result2: serde_json::Value = response2.json().await?;
    let user_data = &result2["data"]["user"];
    println!("[GraphQL Client] 사용자: ID={}, 이름={}", 
        user_data["id"], user_data["name"].as_str().unwrap());

    // 3. 계산 쿼리
    let query3 = r#"{"query": "{ add(a: 15, b: 25) }"}"#;
    println!("[GraphQL Client] 계산 요청 전송");
    
    let response3 = client
        .post("http://127.0.0.1:3000/graphql")
        .header("Content-Type", "application/json")
        .body(query3)
        .send()
        .await?;
    
    let result3: serde_json::Value = response3.json().await?;
    println!("[GraphQL Client] 계산 결과: {}", result3["data"]["add"]);

    Ok(())
}


#[tokio::main]
pub async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(">> GraphQL 예제 시작");

    let graphql_server_handle = tokio::spawn(async {
        if let Err(e) = graphql_server().await {
            println!("[GraphQL Server] 에러: {}", e);
        }
    });

    // 서버 시작 대기
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    if let Err(e) = graphql_client().await {
        println!("[GraphQL Client] 에러: {}", e);
    }

    // 서버 종료
    graphql_server_handle.abort();

    println!("\n완료!");

    Ok(())
}