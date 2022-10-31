use crate::models::User;
use crate::ports::{Database, Hasher, TokenGenerator};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RegistrationError {
    #[error("failed to add user to the database")]
    DatabaseError,
    #[error("requested username already exists")]
    UsernameTaken,
    #[error("token generation error")]
    TokenGenerationError,
}

pub async fn register(
    user: User,
    db: impl Database,
    hasher: impl Hasher,
    token_generator: impl TokenGenerator,
) -> Result<String, RegistrationError> {
    if let Ok(Some(_)) = db.get_user(&user.username).await {
        return Err(RegistrationError::UsernameTaken);
    }

    let hashed_password = hasher.hash_password(&user.password).await;
    let token = token_generator
        .generate(&user.username)
        .or(Err(RegistrationError::TokenGenerationError))?;

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
    #[error("token generation error")]
    TokenGenerationError,
}

pub async fn login(
    user: User,
    db: impl Database,
    hasher: impl Hasher,
    token_generator: impl TokenGenerator,
) -> Result<String, LoginError> {
    let found_user = db
        .get_user(&user.username)
        .await
        .or(Err(LoginError::DatabaseError))?
        .ok_or(LoginError::UserNotFound)?;

    if hasher
        .compare_password(&user.password, &found_user.password.clone())
        .await
    {
        let token = token_generator
            .generate(&user.username)
            .or(Err(LoginError::TokenGenerationError))?;
        Ok(token)
    } else {
        Err(LoginError::IncorrectPassword)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AuthorizationError {
    #[error("failed to get user from the database")]
    DatabaseError,
    #[error("requested user was not found")]
    UserNotFound,
    #[error("invalid token")]
    InvalidToken,
    #[error("token has expired")]
    ExpiredToken,
}

pub async fn authorize(
    token: String,
    db: impl Database,
    token_generator: impl TokenGenerator,
) -> Result<(), AuthorizationError> {
    let claim = token_generator
        .get_claims(token)
        .ok_or(AuthorizationError::InvalidToken)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now > claim.iat + claim.exp {
        return Err(AuthorizationError::ExpiredToken);
    }

    db.get_user(&claim.sub)
        .await
        .or(Err(AuthorizationError::DatabaseError))?
        .ok_or(AuthorizationError::UserNotFound)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{authorize, login, register, LoginError, RegistrationError};
    use crate::{
        adapters::{Jwt, MemoryDatabase, ShaHasher, SimpleTokenGenerator},
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

    #[tokio::test]
    async fn simple_authorization() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let reg_result = register(user.clone(), db.clone(), ShaHasher, Jwt).await;
        assert!(reg_result.is_ok());
        let token = reg_result.unwrap();

        let decoded_user = authorize(token, db, Jwt).await;
        assert!(decoded_user.is_ok());
    }
}
