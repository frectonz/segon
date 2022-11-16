use crate::{
    models::Game,
    ports::{GameDatabase, UserModel, UsersDatabase},
};
use async_trait::async_trait;
use redis::{aio::Connection, Client, Value};
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

    async fn add_user(&self, user: UserModel) -> Result<(), Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        Ok(redis::cmd("HSET")
            .arg(format!("user:{}", user.id()))
            .arg("username")
            .arg(user.username())
            .arg("password")
            .arg(user.password())
            .query_async(&mut *connection)
            .await?)
    }

    async fn get_user(&self, id: &str) -> Result<Option<UserModel>, Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        let data: Value = redis::cmd("HGETALL")
            .arg(format!("user:{}", id))
            .query_async(&mut *connection)
            .await?;

        match data {
            Value::Bulk(values) if values.len() >= 5 => {
                let username = string_from_redis_value(&values[1]);
                let password = string_from_redis_value(&values[3]);

                Ok(username
                    .map(|username| {
                        password.map(|password| UserModel::new(id.into(), username, password))
                    })
                    .flatten())
            }
            _ => Ok(None),
        }
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<UserModel>, Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;

        let data: Value = redis::cmd("FT.SEARCH")
            .arg("usernameIndex")
            .arg(username)
            .arg("LIMIT")
            .arg("0")
            .arg("1")
            .query_async(&mut *connection)
            .await?;

        match data {
            Value::Bulk(values) if values.len() >= 6 => {
                let id = string_from_redis_value(&values[1]);
                let username = string_from_redis_value(&values[3]);
                let password = string_from_redis_value(&values[5]);

                Ok(id
                    .map(|id| {
                        username.map(|username| {
                            password.map(|password| UserModel::new(id, username, password))
                        })
                    })
                    .flatten()
                    .flatten())
            }
            _ => Ok(None),
        }
    }
}

fn string_from_redis_value(v: &Value) -> Option<String> {
    match v {
        Value::Data(d) => String::from_utf8(d.to_vec()).ok(),
        _ => None,
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
