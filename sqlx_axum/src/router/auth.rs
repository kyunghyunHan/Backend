// use std::result;

// use crate::db::connection::connection;
// use crate::model::auth::{JoinUser, LoginUser, User};
// use axum::{response::Json as JsonResponse, Json};
// use serde_json::{json, Value};
// use sqlx::{query, Row};
// pub async fn get_users() -> JsonResponse<Value> {
//     let connection = connection().await;
//     let users = sqlx::query("SELECT id, user_id,user_pw FROM users")
//         .fetch_all(&connection)
//         .await
//         .expect("Failedq to fetch users.");

//     let users: Vec<User> = users
//         .into_iter()
//         .map(|row| User {
//             id: row.get("id"),
//             user_id: row.get("user_id"),
//             user_pw: row.get("user_pw"),
//         })
//         .collect();
//     JsonResponse(json!({ "result": true,"users":users}))
// }

// pub async fn login(input: Json<LoginUser>) -> JsonResponse<Value> {
//     let connection: sqlx::Pool<sqlx::Postgres> = connection().await;
//     let query =
//         sqlx::query("SELECT id, user_id, user_pw FROM users WHERE user_id = $1 AND user_pw = $2")
//             .bind(&input.user_id) // user_id 값 바인딩
//             .bind(&input.user_pw) // user_pw 값 바인딩
//             .fetch_all(&connection)
//             .await
//             .expect("Failed to fetch users.");
//     let result = !query.is_empty();
//     println!("{:?}", result);

//     // let users: Vec<User> = query
//     //     .into_iter()
//     //     .map(|row| User {
//     //         id: row.get("id"),
//     //         user_id: row.get("user_id"),
//     //         user_pw: row.get("user_pw"),
//     //     })
//     //     .collect();
//     JsonResponse(json!({ "result": result}))
// }

// pub async fn join(input: Json<JoinUser>) -> JsonResponse<Value> {
//     let connection: sqlx::Pool<sqlx::Postgres> = connection().await;
//     println!("{:?}", input);

//     let query = sqlx::query("SELECT id, user_id FROM users WHERE user_id = $1 ")
//         .bind(&input.user_id) // user_id 값 바인딩
//         .fetch_all(&connection)
//         .await
//         .expect("Failed to fetch users.");

//     let result = query.is_empty();
//     println!("{}", result);
    
//     match result {
//         true => {
//             let result = sqlx::query("INSERT INTO users (user_id, user_pw, user_name) VALUES ($1, $2 ,$3)")
//                 .bind(&input.user_id) // user_id 값 바인딩
//                 .bind(&input.user_pw) // user_pw 값 바인딩
//                 .bind(&input.user_name) // user_pw 값 바인딩

//                 .execute(&connection) // 쿼리 실행
//                 .await
//                 .expect("Failed to insert user");

//             match result {
//                 exec_result => {
//                     if exec_result.rows_affected() > 0 {
//                         JsonResponse(json!({ "result": true}))
//                         // 추가 작업이 필요할 경우 여기에 작성
//                     } else {
//                         JsonResponse(json!({ "result":false,"msg":"You are already registered"}))
//                     }
//                 }
//             }
//         }
//         false => JsonResponse(json!({ "result":false,"msg":"You are already registered"})),
//     }
// }

// pub async fn secession(input: Json<LoginUser>) -> JsonResponse<Value> {
//     let connection: sqlx::Pool<sqlx::Postgres> = connection().await;
//     println!("{:?}", input);
//     let query = sqlx::query("SELECT user_id FROM users WHERE user_id = $1 AND user_pw = $2")
//         .bind(&input.user_id) // user_id binding
//         .bind(&input.user_pw) // user_pw binding
//         .fetch_all(&connection)
//         .await
//         .expect("Failed to fetch users.");

//     let user_exists = !query.is_empty();
//     println!("{}", user_exists);

//     match user_exists {
//         true => {
//             // Delete user if they exist
//             let result = sqlx::query("DELETE FROM users WHERE user_id = $1 AND user_pw = $2")
//                 .bind(&input.user_id) // user_id binding
//                 .bind(&input.user_pw) // user_pw binding
//                 .execute(&connection) // Execute the delete query
//                 .await
//                 .expect("Failed to delete user");

//             if result.rows_affected() > 0 {
//                 JsonResponse(json!({ "result": true, "msg": "User deleted successfully."}))
//             } else {
//                 JsonResponse(json!({ "result": false, "msg": "Failed to delete the user."}))
//             }
//         }
//         false => {
//             JsonResponse(json!({ "result": false, "msg": "User not found or password incorrect."}))
//         }
//     }
// }
// pub async fn update_user(input: Json<JoinUser>) -> JsonResponse<Value> {
//     let connection: sqlx::Pool<sqlx::Postgres> = connection().await;
//     println!("{:?}", input);

//     // 사용자 존재 여부 확인
//     let query = sqlx::query("SELECT user_id FROM users WHERE user_id = $1 AND user_pw = $2")
//         .bind(&input.user_id) // user_id 바인딩
//         .bind(&input.user_pw) // user_pw 바인딩
//         .fetch_all(&connection)
//         .await
//         .expect("Failed to fetch users.");

//     let user_exists = !query.is_empty();
//     println!("{}", user_exists);

//     match user_exists {
//         true => {
//             // 사용자가 존재할 경우 업데이트 작업 수행 (예: user_pw를 새 비밀번호로 변경)
//             let new_name = &input.user_name; // 새 비밀번호 또는 업데이트할 값

//             let result =
//                 sqlx::query("UPDATE users SET user_name = $1 WHERE user_id = $2 AND user_pw = $3")
//                     .bind(new_name) // 새로운 값 바인딩 (예: 새 비밀번호)
//                     .bind(&input.user_id) // user_id 바인딩
//                     .bind(&input.user_pw) // 기존 비밀번호 확인용 바인딩
//                     .execute(&connection) // 업데이트 쿼리 실행
//                     .await
//                     .expect("Failed to update user.");

//             if result.rows_affected() > 0 {
//                 JsonResponse(json!({ "result": true, "msg": "User updated successfully." }))
//             } else {
//                 JsonResponse(json!({ "result": false, "msg": "Failed to update the user." }))
//             }
//         }
//         false => {
//             JsonResponse(json!({ "result": false, "msg": "User not found or password incorrect." }))
//         }
//     }
// }
