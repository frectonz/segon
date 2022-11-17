use crate::models::{AnswerStatus, Game, OptionIndex, Question};
use crate::ports::{GameDatabase, UserModel, UsersDatabase};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct UsersMemoryDatabase {
    users: Arc<Mutex<Vec<UserModel>>>,
    fail: bool,
}

impl UsersMemoryDatabase {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            fail: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            fail: true,
        }
    }
}

#[derive(Error, Debug)]
pub enum UsersMemoryDatabaseError {
    #[error("failed to add user to database")]
    AddUserError,
    #[error("failed to get user from database")]
    GetUserError,
}

#[async_trait]
impl UsersDatabase for UsersMemoryDatabase {
    type Error = UsersMemoryDatabaseError;

    async fn add_user(&self, user: UserModel) -> Result<(), Self::Error> {
        if self.fail {
            return Err(UsersMemoryDatabaseError::AddUserError);
        }

        let mut users = self.users.lock().await;
        users.push(user);
        Ok(())
    }

    async fn get_user(&self, id: &str) -> Result<Option<UserModel>, Self::Error> {
        if self.fail {
            return Err(UsersMemoryDatabaseError::GetUserError);
        }

        let users = self.users.lock().await;
        Ok(users.iter().find(|user| user.id() == id).cloned())
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<UserModel>, Self::Error> {
        if self.fail {
            return Err(UsersMemoryDatabaseError::GetUserError);
        }

        let users = self.users.lock().await;
        Ok(users
            .iter()
            .find(|user| user.username() == username)
            .cloned())
    }
}
#[derive(Debug, Clone, Default)]
pub struct GameMemoryDatabase {
    answers: Arc<Mutex<HashMap<(String, String), OptionIndex>>>,
    answer_statuses: Arc<Mutex<HashMap<(String, String), AnswerStatus>>>,
    scores: Arc<Mutex<HashMap<String, u32>>>,
}

#[async_trait]
impl GameDatabase for GameMemoryDatabase {
    type Error = std::convert::Infallible;

    async fn get_game(&self) -> Result<Option<Game>, Self::Error> {
        Ok(Some(Game {
            questions: vec![
                Question {
                    question: "What is question 1?".into(),
                    options: [
                        "Option 1".into(),
                        "Option 2".into(),
                        "Option 3".into(),
                        "Option 4".into(),
                    ],
                    answer_idx: OptionIndex::One,
                },
                Question {
                    question: "What is question 2?".into(),
                    options: [
                        "Option 1".into(),
                        "Option 2".into(),
                        "Option 3".into(),
                        "Option 4".into(),
                    ],
                    answer_idx: OptionIndex::One,
                },
            ],
        }))
    }

    async fn set_answer(
        &self,
        id: &str,
        question: &str,
        answer: OptionIndex,
    ) -> Result<(), Self::Error> {
        let mut answers = self.answers.lock().await;
        answers.insert((id.into(), question.into()), answer);
        Ok(())
    }

    async fn get_answer(
        &self,
        id: &str,
        question: &str,
    ) -> Result<Option<OptionIndex>, Self::Error> {
        let answers = self.answers.lock().await;
        Ok(answers.get(&(id.into(), question.into())).cloned())
    }

    async fn set_answer_status(
        &self,
        id: &str,
        question: &str,
        answer_status: &AnswerStatus,
    ) -> Result<(), Self::Error> {
        let mut answer_statuses = self.answer_statuses.lock().await;
        answer_statuses.insert((id.into(), question.into()), answer_status.clone());
        Ok(())
    }

    async fn get_answers_statuses(&self, username: &str) -> Result<Vec<AnswerStatus>, Self::Error> {
        let answer_statuses = self.answer_statuses.lock().await;
        Ok(answer_statuses
            .iter()
            .filter(|((user, _), _)| user == username)
            .map(|((_, _), answer_status)| answer_status.clone())
            .collect())
    }

    async fn set_score(&self, username: &str, score: u32) -> Result<(), Self::Error> {
        let mut scores = self.scores.lock().await;
        scores.insert(username.into(), score);
        Ok(())
    }
}
