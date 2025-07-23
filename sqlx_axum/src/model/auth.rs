use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub user_id: String,
    pub user_pw: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinUser {
    pub user_id: String,
    pub user_pw: String,
    pub user_name: String,
}
#[derive(Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub user_pw: String,
}
