use crate::models::{AnswerStatus, Game, OptionIndex};
use async_trait::async_trait;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct UserModel {
    id: String,
    username: String,
    password: String,
}

impl UserModel {
    pub fn new(id: String, username: String, password: String) -> Self {
        Self {
            id,
            username,
            password,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[async_trait]
pub trait UsersDatabase {
    type Error: Error + Send + Sync + 'static;
    async fn add_user(&self, user: UserModel) -> Result<(), Self::Error>;
    async fn get_user(&self, id: &str) -> Result<Option<UserModel>, Self::Error>;
    async fn get_by_username(&self, username: &str) -> Result<Option<UserModel>, Self::Error>;
}

#[async_trait]
pub trait GameDatabase {
    type Error;
    async fn get_game(&self) -> Game;
    async fn set_answer(
        &self,
        id: &str,
        question: &str,
        answer: OptionIndex,
    ) -> Result<(), Self::Error>;
    async fn get_answer(&self, id: &str, question: &str) -> Option<OptionIndex>;
    async fn set_answer_status(
        &self,
        id: &str,
        question: &str,
        answer_status: &AnswerStatus,
    ) -> Result<(), Self::Error>;
    async fn get_answers_statuses(&self, id: &str) -> Result<Vec<AnswerStatus>, Self::Error>;
    async fn set_score(&self, id: &str, score: u32) -> Result<(), Self::Error>;
}
