use crate::models::User;
use crate::ports::{Hasher, TokenGenerator, UsersDatabase};
use thiserror::Error;

#[derive(Clone)]
pub struct UsersController<D, H, T>
where
    D: UsersDatabase,
    H: Hasher,
    T: TokenGenerator,
{
    db: D,
    hasher: H,
    token_generator: T,
}

impl<D, H, T> UsersController<D, H, T>
where
    D: UsersDatabase,
    H: Hasher,
    T: TokenGenerator,
{
    pub fn new(db: D, hasher: H, token_generator: T) -> Self {
        Self {
            db,
            hasher,
            token_generator,
        }
    }

    pub async fn register(&self, user: User) -> Result<String, RegistrationError> {
        if let Ok(Some(_)) = self.db.get_user(&user.username).await {
            return Err(RegistrationError::UsernameTaken);
        }

        let hashed_password = self.hasher.hash_password(&user.password).await;
        let token = self
            .token_generator
            .generate(&user.username)
            .or(Err(RegistrationError::TokenGenerationError))?;

        self.db
            .add_user(User {
                username: user.username,
                password: hashed_password,
            })
            .await
            .or(Err(RegistrationError::DatabaseError))?;
        Ok(token)
    }

    pub async fn login(&self, user: User) -> Result<String, LoginError> {
        let found_user = self
            .db
            .get_user(&user.username)
            .await
            .or(Err(LoginError::DatabaseError))?
            .ok_or(LoginError::UserNotFound)?;

        if self
            .hasher
            .compare_password(&user.password, &found_user.password.clone())
            .await
        {
            let token = self
                .token_generator
                .generate(&user.username)
                .or(Err(LoginError::TokenGenerationError))?;
            Ok(token)
        } else {
            Err(LoginError::IncorrectPassword)
        }
    }

    pub async fn authorize(&self, token: String) -> Result<User, AuthorizationError> {
        let claim = self
            .token_generator
            .get_claims(token)
            .ok_or(AuthorizationError::InvalidToken)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > claim.iat + claim.exp {
            return Err(AuthorizationError::ExpiredToken);
        }

        let user = self
            .db
            .get_user(&claim.sub)
            .await
            .or(Err(AuthorizationError::DatabaseError))?
            .ok_or(AuthorizationError::UserNotFound)?;

        Ok(user)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RegistrationError {
    #[error("failed to add user to the database")]
    DatabaseError,
    #[error("requested username already exists")]
    UsernameTaken,
    #[error("token generation error")]
    TokenGenerationError,
}

#[derive(Error, Debug, PartialEq, Eq)]
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

#[derive(Error, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::{LoginError, RegistrationError, UsersController};
    use crate::{
        adapters::{Jwt, ShaHasher, UsersMemoryDatabase},
        models::User,
    };

    fn sample_user() -> User {
        User {
            username: "Frectonz".into(),
            password: "123".into(),
        }
    }

    fn get_controller() -> UsersController<UsersMemoryDatabase, ShaHasher, Jwt> {
        UsersController::new(UsersMemoryDatabase::new(), ShaHasher, Jwt)
    }

    fn get_failing_controller() -> UsersController<UsersMemoryDatabase, ShaHasher, Jwt> {
        UsersController::new(UsersMemoryDatabase::failing(), ShaHasher, Jwt)
    }

    #[tokio::test]
    async fn simple_registration() {
        let user = sample_user();
        let controller = get_controller();

        let res = controller.register(user).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn failing_registration() {
        let user = sample_user();
        let controller = get_failing_controller();

        let res = controller.register(user).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(RegistrationError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_register_with_an_existing_username() {
        let user = sample_user();
        let controller = get_controller();

        let res = controller.register(user.clone()).await;
        assert!(res.is_ok());

        let res = controller.register(user).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(RegistrationError::UsernameTaken));
    }

    #[tokio::test]
    async fn register_and_login() {
        let user = sample_user();
        let controller = get_controller();

        let res = controller.register(user.clone()).await;
        assert!(res.is_ok());

        let res = controller.login(user).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn failing_login() {
        let user = sample_user();
        let controller = get_failing_controller();

        let res = controller.login(user).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(LoginError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_login_to_a_non_existent_account() {
        let user = sample_user();
        let controller = get_controller();

        let res = controller.login(user).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(LoginError::UserNotFound));
    }

    #[tokio::test]
    async fn can_not_login_with_incorrect_password() {
        let mut user = sample_user();
        let controller = get_controller();

        let res = controller.register(user.clone()).await;
        assert!(res.is_ok());

        user.password = "wrong".into();

        let res = controller.login(user).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(LoginError::IncorrectPassword));
    }

    #[tokio::test]
    async fn simple_authorization() {
        let user = sample_user();
        let controller = get_controller();

        let res = controller.register(user).await;
        assert!(res.is_ok());

        let token = res.unwrap();
        let decoded_user = controller.authorize(token).await;
        assert!(decoded_user.is_ok());
    }
}
