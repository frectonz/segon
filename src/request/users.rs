use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

impl RegisterRequest {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn set_password(&mut self, password: &str) {
        self.password = password.into();
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RegisterValidationError {
    #[error("username is empty")]
    UsernameEmpty,
    #[error("password is empty")]
    PasswordEmpty,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), RegisterValidationError> {
        use RegisterValidationError::*;

        if self.username.is_empty() {
            return Err(UsernameEmpty);
        }

        if self.password.is_empty() {
            return Err(PasswordEmpty);
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LoginRequest {
    username: String,
    password: String,
}

impl LoginRequest {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LoginValidationError {
    #[error("username is empty")]
    UsernameEmpty,
    #[error("password is empty")]
    PasswordEmpty,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), LoginValidationError> {
        use LoginValidationError::*;

        if self.username.is_empty() {
            return Err(UsernameEmpty);
        }

        if self.password.is_empty() {
            return Err(PasswordEmpty);
        }

        Ok(())
    }
}

impl From<RegisterRequest> for LoginRequest {
    fn from(request: RegisterRequest) -> Self {
        Self::new(request.username, request.password)
    }
}
