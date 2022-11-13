use crate::models::{Game, OptionIndex, Question, User};
use crate::ports::{GameDatabase, UsersDatabase};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MemoryDatabase {
    users: Arc<Mutex<Vec<User>>>,
    fail: bool,
    answers: Arc<Mutex<HashMap<(String, String), OptionIndex>>>,
}

impl MemoryDatabase {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            fail: false,
            answers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn failing() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            fail: true,
            answers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_users(users: Vec<User>) -> Self {
        Self {
            users: Arc::new(Mutex::new(users)),
            fail: false,
            answers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Error, Debug)]
pub enum MemoryDatabaseError {
    #[error("failed to add user to database")]
    AddUserError,
    #[error("failed to get user from database")]
    GetUserError,
}

#[async_trait]
impl UsersDatabase for MemoryDatabase {
    type Error = MemoryDatabaseError;

    async fn add_user(&self, user: User) -> Result<(), Self::Error> {
        if self.fail {
            return Err(MemoryDatabaseError::AddUserError);
        }

        let mut users = self.users.lock().await;
        users.push(user);
        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<User>, Self::Error> {
        if self.fail {
            return Err(MemoryDatabaseError::GetUserError);
        }

        let users = self.users.lock().await;
        Ok(users
            .clone()
            .into_iter()
            .find(|user| user.username == username))
    }
}

#[async_trait]
impl GameDatabase for MemoryDatabase {
    type Error = String;

    async fn get_game(&self) -> Game {
        Game {
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
        }
    }

    async fn set_answer(
        &self,
        username: &str,
        question: &str,
        answer: OptionIndex,
    ) -> Result<(), Self::Error> {
        let mut answers = self.answers.lock().await;
        answers.insert((username.into(), question.into()), answer);
        Ok(())
    }

    async fn get_answer(&self, username: &str, question: &str) -> Option<OptionIndex> {
        let answers = self.answers.lock().await;
        answers.get(&(username.into(), question.into())).cloned()
    }
}
