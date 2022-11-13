use crate::models::{AnswerStatus, Game, OptionIndex, User};
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
    async fn set_answer(
        &self,
        username: &str,
        question: &str,
        answer: OptionIndex,
    ) -> Result<(), Self::Error>;
    async fn get_answer(&self, username: &str, question: &str) -> Option<OptionIndex>;
    async fn set_answer_status(
        &self,
        username: &str,
        question: &str,
        answer_status: &AnswerStatus,
    ) -> Result<(), Self::Error>;

    async fn get_answers_statuses(&self, username: &str) -> Result<Vec<AnswerStatus>, Self::Error>;

    async fn set_score(&self, username: &str, score: u32) -> Result<(), Self::Error>;
}
