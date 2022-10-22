use crate::models::User;
use crate::ports::Database;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MemoryDatabase {
    users: Arc<Mutex<Vec<User>>>,
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Database for MemoryDatabase {
    async fn add_user(&self, user: User) -> Result<(), Box<dyn std::error::Error>> {
        let mut users = self.users.lock().await;
        users.push(user);
        Ok(())
    }

    async fn get_user(&self, username: String) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let users = self.users.lock().await;
        Ok(users
            .clone()
            .into_iter()
            .find(|user| user.username == username))
    }
}
