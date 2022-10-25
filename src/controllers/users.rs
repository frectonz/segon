use crate::models::User;
use crate::ports::{Database, Hasher, TokenGenerator};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RegistrationError {
    #[error("failed to add user to the database")]
    DatabaseError,
    #[error("requested username already exists")]
    UsernameTaken,
}

pub async fn register<
    Token: Display + Serialize + for<'a> Deserialize<'a>,
    TG: TokenGenerator<Token>,
>(
    user: User,
    db: impl Database,
    hasher: impl Hasher,
    token_generator: TG,
) -> Result<Token, RegistrationError> {
    if let Ok(Some(_)) = db.get_user(&user.username).await {
        return Err(RegistrationError::UsernameTaken);
    }

    let hashed_password = hasher.hash_password(&user.password).await;
    let token = token_generator.generate(&user.username);

    db.add_user(User {
        username: user.username,
        password: hashed_password,
    })
    .await
    .or(Err(RegistrationError::DatabaseError))?;

    Ok(token)
}

#[derive(Error, Debug, PartialEq)]
pub enum LoginError {
    #[error("failed to get user from the database")]
    DatabaseError,
    #[error("requested user was not found")]
    UserNotFound,
    #[error("the provided password is incorrect")]
    IncorrectPassword,
}

pub async fn login<
    Token: Display + Serialize + for<'a> Deserialize<'a>,
    TG: TokenGenerator<Token>,
>(
    user: User,
    db: impl Database,
    hasher: impl Hasher,
    token_generator: TG,
) -> Result<Token, LoginError> {
    let found_user = db
        .get_user(&user.username)
        .await
        .or(Err(LoginError::DatabaseError))?
        .ok_or(LoginError::UserNotFound)?;

    if hasher
        .compare_password(&user.password, &found_user.password.clone())
        .await
    {
        let token = token_generator.generate(&user.username);
        Ok(token)
    } else {
        Err(LoginError::IncorrectPassword)
    }
}

#[cfg(test)]
mod tests {
    use super::{login, register, LoginError, RegistrationError};
    use crate::{
        adapters::{MemoryDatabase, ShaHasher, SimpleTokenGenerator},
        models::User,
    };

    fn sample_user() -> User {
        User {
            username: "Frectonz".into(),
            password: "123".into(),
        }
    }

    #[tokio::test]
    async fn simple_registration() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let reg_result = register(user.clone(), db.clone(), ShaHasher, SimpleTokenGenerator).await;
        assert!(reg_result.is_ok());
    }

    #[tokio::test]
    async fn failing_registration() {
        let db = MemoryDatabase::failing();
        let user = sample_user();

        let reg_result = register(user.clone(), db.clone(), ShaHasher, SimpleTokenGenerator).await;
        assert!(reg_result.is_err());
        assert_eq!(reg_result.err(), Some(RegistrationError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_register_with_an_existing_username() {
        let db = MemoryDatabase::new();
        let user = sample_user();
        let reg_result = register(user.clone(), db.clone(), ShaHasher, SimpleTokenGenerator).await;
        assert!(reg_result.is_ok());

        let reg_result2 = register(user.clone(), db.clone(), ShaHasher, SimpleTokenGenerator).await;
        assert!(reg_result2.is_err());
        assert_eq!(reg_result2.err(), Some(RegistrationError::UsernameTaken));
    }

    #[tokio::test]
    async fn register_and_login() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let reg_result = register(user.clone(), db.clone(), ShaHasher, SimpleTokenGenerator).await;
        assert!(reg_result.is_ok());

        let login_result = login(user, db, ShaHasher, SimpleTokenGenerator).await;
        assert!(login_result.is_ok());
    }

    #[tokio::test]
    async fn failing_login() {
        let db = MemoryDatabase::failing();
        let user = sample_user();

        let login_result = login(user, db, ShaHasher, SimpleTokenGenerator).await;
        assert!(login_result.is_err());
        assert_eq!(login_result.err(), Some(LoginError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_login_to_a_non_existent_account() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let login_result = login(user, db, ShaHasher, SimpleTokenGenerator).await;
        assert!(login_result.is_err());
        assert_eq!(login_result.err(), Some(LoginError::UserNotFound));
    }

    #[tokio::test]
    async fn can_not_login_with_incorrect_password() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let reg_result = register(user, db.clone(), ShaHasher, SimpleTokenGenerator).await;
        assert!(reg_result.is_ok());

        let login_result = login(
            User {
                username: "Frectonz".into(),
                password: "wrong".into(),
            },
            db,
            ShaHasher,
            SimpleTokenGenerator,
        )
        .await;
        assert!(login_result.is_err());
        assert_eq!(login_result.err(), Some(LoginError::IncorrectPassword));
    }
}
