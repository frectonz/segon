use crate::models::User;
use async_trait::async_trait;

#[async_trait]
pub trait Database {
    async fn add_user<'a>(&'a self, user: User) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_user<'a>(
        &'a self,
        username: String,
    ) -> Result<Option<User>, Box<dyn std::error::Error>>;
}
