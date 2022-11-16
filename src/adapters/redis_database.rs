use crate::{
    models::{Game, User},
    ports::{GameDatabase, UsersDatabase},
};
use async_trait::async_trait;
use redis::{aio::Connection, Client};
use std::sync::Arc;
use tokio::sync::Mutex;

const REDIS_CONNECTION_STRING: &str = "redis://localhost:6379";

#[derive(Clone)]
pub struct RedisUsersDatabase {
    connection: Arc<Mutex<Connection>>,
}

impl RedisUsersDatabase {
    pub async fn new() -> Self {
        let client = Client::open(REDIS_CONNECTION_STRING).unwrap();
        let connection = client.get_async_connection().await.unwrap();
        let connection = Arc::new(Mutex::new(connection));

        Self { connection }
    }
}

#[async_trait]
impl UsersDatabase for RedisUsersDatabase {
    type Error = redis::RedisError;

    async fn add_user(&self, user: User) -> Result<(), Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        let _: () = redis::cmd("SET")
            .arg(user.username)
            .arg(user.password)
            .query_async(&mut *connection)
            .await?;

        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<User>, Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;

        let password: Option<String> = redis::cmd("GET")
            .arg(username)
            .query_async(&mut *connection)
            .await?;

        Ok(password.map(|password| User {
            username: username.to_string(),
            password,
        }))
    }
}

#[async_trait]
impl GameDatabase for RedisUsersDatabase {
    type Error = redis::RedisError;

    async fn get_game(&self) -> Game {
        todo!()
    }

    async fn set_answer(
        &self,
        _username: &str,
        _question: &str,
        _answer: crate::models::OptionIndex,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_answer(
        &self,
        _username: &str,
        _question: &str,
    ) -> Option<crate::models::OptionIndex> {
        todo!()
    }

    async fn set_answer_status(
        &self,
        _username: &str,
        _question: &str,
        _answer_status: &crate::models::AnswerStatus,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_answers_statuses(
        &self,
        _username: &str,
    ) -> Result<Vec<crate::models::AnswerStatus>, Self::Error> {
        todo!()
    }

    async fn set_score(&self, _username: &str, _score: u32) -> Result<(), Self::Error> {
        todo!()
    }
}
