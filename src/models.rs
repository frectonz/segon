use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}
