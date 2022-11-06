use crate::models::{Game, OptionIndex, Question, User};
use crate::ports::{GameDatabase, UsersDatabase};
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MemoryDatabase {
    users: Arc<Mutex<Vec<User>>>,
    fail: bool,
}

impl MemoryDatabase {
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

    pub fn with_users(users: Vec<User>) -> Self {
        Self {
            users: Arc::new(Mutex::new(users)),
            fail: false,
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
        let game = Game {
            questions: vec![Question {
                question: "What is ___".into(),
                options: [
                    "Option 1".into(),
                    "Option 2".into(),
                    "Option 3".into(),
                    "Option 4".into(),
                ],
                answer_idx: OptionIndex::One,
            }],
        };

        game
    }
}
