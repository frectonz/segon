use crate::models::User;
use crate::ports::{Database, Hasher};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RegistrationError {
    #[error("failed to add user to the database")]
    DatabaseError,
    #[error("requested username already exists")]
    UsernameTaken,
}

pub async fn register(
    user: User,
    db: impl Database,
    hasher: impl Hasher,
) -> Result<(), RegistrationError> {
    let u = user.clone();
    if let Ok(Some(_)) = db.get_user(u.username).await {
        return Err(RegistrationError::UsernameTaken);
    }

    let hashed_password = hasher.hash_password(user.password).await;

    db.add_user(User {
        username: user.username,
        password: hashed_password,
    })
    .await
    .or(Err(RegistrationError::DatabaseError))?;

    Ok(())
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

pub async fn login(user: User, db: impl Database, hasher: impl Hasher) -> Result<User, LoginError> {
    let u = user.clone();
    let found_user = db
        .get_user(user.username)
        .await
        .or(Err(LoginError::DatabaseError))?
        .ok_or(LoginError::UserNotFound)?;

    if hasher
        .compare_password(user.password, found_user.password.clone())
        .await
    {
        Ok(u)
    } else {
        Err(LoginError::IncorrectPassword)
    }
}

#[cfg(test)]
mod tests {
    use super::{login, register, LoginError, RegistrationError};
    use crate::{
        adapters::{MemoryDatabase, ShaHasher},
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

        let reg_result = register(user.clone(), db.clone(), ShaHasher::default()).await;
        assert!(reg_result.is_ok());
    }

    #[tokio::test]
    async fn failing_registration() {
        let db = MemoryDatabase::failing();
        let user = sample_user();

        let reg_result = register(user.clone(), db.clone(), ShaHasher::default()).await;
        assert!(reg_result.is_err());
        assert_eq!(reg_result.err(), Some(RegistrationError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_register_with_an_existing_username() {
        let db = MemoryDatabase::new();
        let user = sample_user();
        let reg_result = register(user.clone(), db.clone(), ShaHasher::default()).await;
        assert!(reg_result.is_ok());

        let reg_result2 = register(user.clone(), db.clone(), ShaHasher::default()).await;
        assert!(reg_result2.is_err());
        assert_eq!(reg_result2.err(), Some(RegistrationError::UsernameTaken));
    }

    #[tokio::test]
    async fn register_and_login() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let reg_result = register(user.clone(), db.clone(), ShaHasher::default()).await;
        assert!(reg_result.is_ok());

        let login_result = login(user, db, ShaHasher::default()).await;
        assert!(login_result.is_ok());
        assert_eq!(login_result.ok(), Some(sample_user()));
    }

    #[tokio::test]
    async fn failing_login() {
        let db = MemoryDatabase::failing();
        let user = sample_user();

        let login_result = login(user, db, ShaHasher::default()).await;
        assert!(login_result.is_err());
        assert_eq!(login_result.err(), Some(LoginError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_login_to_a_non_existent_account() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let login_result = login(user, db, ShaHasher::default()).await;
        assert!(login_result.is_err());
        assert_eq!(login_result.err(), Some(LoginError::UserNotFound));
    }

    #[tokio::test]
    async fn can_not_login_with_incorrect_password() {
        let db = MemoryDatabase::new();
        let user = sample_user();

        let reg_result = register(user, db.clone(), ShaHasher::default()).await;
        assert!(reg_result.is_ok());

        let login_result = login(
            User {
                username: "Frectonz".into(),
                password: "wrong".into(),
            },
            db,
            ShaHasher::default(),
        )
        .await;
        assert!(login_result.is_err());
        assert_eq!(login_result.err(), Some(LoginError::IncorrectPassword));
    }
}
