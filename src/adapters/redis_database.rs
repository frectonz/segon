use crate::{
    models::{AnswerStatus, Game},
    ports::{GameDatabase, UserModel, UsersDatabase},
};
use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client, JsonAsyncCommands, Value};
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
            Value::Bulk(values) if values.len() == 4 => {
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
            Value::Bulk(values) if values.len() == 6 => {
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

    async fn get_game(&self) -> Result<Option<Game>, Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;

        let data: Value = connection.json_get("game:latest", ".").await.unwrap();

        match data {
            Value::Data(data) => {
                let data = String::from_utf8(data.to_vec()).ok();
                Ok(data.map(|data| serde_json::from_str(&data).ok()).flatten())
            }
            _ => Ok(None),
        }
    }

    async fn set_answer(
        &self,
        id: &str,
        question: &str,
        answer: crate::models::OptionIndex,
    ) -> Result<(), Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        let answer = serde_json::to_string(&answer).unwrap();
        connection
            .set(format!("answer:{id}:{question}"), answer)
            .await?;
        Ok(())
    }

    async fn get_answer(
        &self,
        id: &str,
        question: &str,
    ) -> Result<Option<crate::models::OptionIndex>, Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        let answer: Option<String> = connection.get(format!("answer:{id}:{question}")).await?;
        Ok(answer
            .map(|answer| serde_json::from_str(&answer).ok())
            .flatten())
    }

    async fn set_answer_status(
        &self,
        id: &str,
        question: &str,
        answer_status: &crate::models::AnswerStatus,
    ) -> Result<(), Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        let answer_status = serde_json::to_string(&answer_status).unwrap();
        connection
            .set(format!("answer_status:{id}:{question}"), answer_status)
            .await?;
        Ok(())
    }

    async fn get_answers_statuses(&self, id: &str) -> Result<Vec<AnswerStatus>, Self::Error> {
        let c = self.connection.clone();
        let mut c = c.lock().await;
        let answer_statuses: Vec<String> = redis::cmd("KEYS")
            .arg(format!("answer_status:{id}:*"))
            .query_async(&mut *c)
            .await?;

        let mut res = Vec::with_capacity(answer_statuses.len());

        for answer_status in answer_statuses {
            let val: Option<String> = c.get(answer_status).await.unwrap();
            let val: Option<AnswerStatus> =
                val.map(|val| serde_json::from_str(&val).ok()).flatten();

            if val.is_some() {
                res.push(val.unwrap());
            }
        }

        Ok(res)
    }

    async fn set_score(&self, id: &str, score: u32) -> Result<(), Self::Error> {
        let connection = self.connection.clone();
        let mut connection = connection.lock().await;
        connection
            .set(format!("score:{id}"), score.to_string())
            .await?;
        Ok(())
    }
}
