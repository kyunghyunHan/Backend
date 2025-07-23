// use crate::db::connection::connection;
// use crate::model::test::Test;
// use axum::response::Json as JsonResponse;
// use serde_json::{json, Value};
// use sqlx::Row;
// pub async fn test() -> JsonResponse<Value> {
//     let connection = connection().await;
//     let users =
//         sqlx::query("SELECT animal_id FROM animals_ins WHERE name IS NULL OR name = '' OR TRIM(name) = '' ORDER BY animal_id")
//             .fetch_all(&connection)
//             .await
//             .expect("Failed to fetch users.");

//     // let users =
//     //     sqlx::query("SELECT animal_id FROM animals_ins")
//     //         .fetch_all(&connection)
//     //         .await
//     //         .expect("Failed to fetch users.");
//     println!("{:?}", users);
//     let animal_ins: Vec<Test> = users
//         .into_iter()
//         .map(|row| Test {
//             animal_ins: row.get("animal_id"),
//         })
//         .collect();
//     JsonResponse(json!({ "result": true,"animal_ins":animal_ins}))
// }
