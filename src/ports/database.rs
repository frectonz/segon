use crate::models::{Game, User};
use async_trait::async_trait;

#[async_trait]
pub trait UsersDatabase {
    type Error;
    async fn add_user(&self, user: User) -> Result<(), Self::Error>;
    async fn get_user(&self, username: &str) -> Result<Option<User>, Self::Error>;
}

#[async_trait]
pub trait GameDatabase {
    type Error;
    async fn get_game(&self) -> Game;
}
