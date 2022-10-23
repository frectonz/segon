use crate::models::User;
use async_trait::async_trait;

#[async_trait]
pub trait Database {
    type Error;
    async fn add_user<'a>(&'a self, user: User) -> Result<(), Self::Error>;
    async fn get_user<'a>(&'a self, username: String) -> Result<Option<User>, Self::Error>;
}
