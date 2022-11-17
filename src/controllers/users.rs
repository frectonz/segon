use crate::{
    ports::{Hasher, IDGenerator, TokenGenerator, UserModel, UsersDatabase},
    request::{LoginRequest, LoginValidationError, RegisterRequest, RegisterValidationError},
};
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Clone)]
pub struct UsersController<D, H, T, I>
where
    D: UsersDatabase,
    H: Hasher,
    T: TokenGenerator,
    I: IDGenerator,
{
    db: D,
    _data: (PhantomData<H>, PhantomData<T>, PhantomData<I>),
}

impl<D, H, T, I> UsersController<D, H, T, I>
where
    D: UsersDatabase,
    H: Hasher,
    T: TokenGenerator,
    I: IDGenerator,
{
    pub fn new(db: D) -> Self {
        Self {
            db,
            _data: (PhantomData, PhantomData, PhantomData),
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<String, RegistrationError> {
        use RegistrationError::*;
        request.validate()?;

        let user = self
            .db
            .get_by_username(request.username())
            .await
            .or(Err(DatabaseError))?;

        if user.is_some() {
            return Err(UsernameTaken);
        }

        let id = I::generate().await;
        let username = request.username().into();
        let hashed_password = H::hash_password(request.password().into())
            .await
            .or(Err(HashError))?;
        let model = UserModel::new(id.clone(), username, hashed_password);

        self.db.add_user(model).await.or(Err(DatabaseError))?;

        T::generate(id).await.or(Err(TokenGenerationError))
    }

    pub async fn login(&self, request: LoginRequest) -> Result<String, LoginError> {
        use LoginError::*;
        request.validate()?;

        let user = self
            .db
            .get_by_username(request.username())
            .await
            .unwrap()
            .ok_or(UserNotFound)?;

        if H::compare_password(request.password().into(), user.password().into())
            .await
            .or(Err(HashCompareError))?
        {
            T::generate(user.id().into())
                .await
                .or(Err(TokenGenerationError))
        } else {
            Err(IncorrectPassword)
        }
    }

    pub async fn authorize(&self, token: String) -> Result<String, AuthorizationError> {
        use AuthorizationError::*;
        let claim = T::get_claims(token).await.or(Err(InvalidToken))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > claim.iat() + claim.exp() {
            return Err(ExpiredToken);
        }

        let user = self
            .db
            .get_user(claim.sub())
            .await
            .or(Err(DatabaseError))?
            .ok_or(UserNotFound)?;

        Ok(user.id().into())
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RegistrationError {
    #[error("failed to add user to the database")]
    DatabaseError,
    #[error("requested username already exists")]
    UsernameTaken,
    #[error("error while hashing the password")]
    HashError,
    #[error("token generation error")]
    TokenGenerationError,
    #[error("validation error")]
    RequestValidationError(#[from] RegisterValidationError),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LoginError {
    #[error("failed to get user from the database")]
    DatabaseError,
    #[error("requested user was not found")]
    UserNotFound,
    #[error("the provided password is incorrect")]
    IncorrectPassword,
    #[error("error while comparing the password")]
    HashCompareError,
    #[error("token generation error")]
    TokenGenerationError,
    #[error("validation error")]
    RequestValidationError(#[from] LoginValidationError),
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
        adapters::{Jwt, ShaHasher, UsersMemoryDatabase, UuidGenerator},
        request::{LoginRequest, RegisterRequest},
    };

    fn sample_register_request() -> RegisterRequest {
        RegisterRequest::new("test".into(), "123".into())
    }

    fn sample_login_request() -> LoginRequest {
        LoginRequest::new("test".into(), "123".into())
    }

    fn get_controller() -> UsersController<UsersMemoryDatabase, ShaHasher, Jwt, UuidGenerator> {
        UsersController::new(UsersMemoryDatabase::new())
    }

    fn get_failing_controller(
    ) -> UsersController<UsersMemoryDatabase, ShaHasher, Jwt, UuidGenerator> {
        UsersController::new(UsersMemoryDatabase::failing())
    }

    #[tokio::test]
    async fn simple_registration() {
        let register_request = sample_register_request();
        let controller = get_controller();

        let res = controller.register(register_request).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn failing_registration() {
        let register_request = sample_register_request();
        let controller = get_failing_controller();

        let res = controller.register(register_request).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(RegistrationError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_register_with_an_existing_username() {
        let register_request = sample_register_request();
        let controller = get_controller();

        let res = controller.register(register_request.clone()).await;
        assert!(res.is_ok());

        let res = controller.register(register_request).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(RegistrationError::UsernameTaken));
    }

    #[tokio::test]
    async fn register_and_login() {
        let register_request = sample_register_request();
        let controller = get_controller();

        let res = controller.register(register_request.clone()).await;
        assert!(res.is_ok());

        let login_request = register_request.into();
        let res = controller.login(login_request).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn failing_login() {
        let login_request = sample_login_request();
        let controller = get_failing_controller();

        let res = controller.login(login_request).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(LoginError::DatabaseError));
    }

    #[tokio::test]
    async fn can_not_login_to_a_non_existent_account() {
        let login_request = sample_login_request();
        let controller = get_controller();

        let res = controller.login(login_request).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(LoginError::UserNotFound));
    }

    #[tokio::test]
    async fn can_not_login_with_incorrect_password() {
        let mut register_request = sample_register_request();
        let controller = get_controller();

        let res = controller.register(register_request.clone()).await;
        assert!(res.is_ok());

        register_request.set_password("wrong");

        let login_request = register_request.into();
        let res = controller.login(login_request).await;
        assert!(res.is_err());
        assert_eq!(res.err(), Some(LoginError::IncorrectPassword));
    }

    #[tokio::test]
    async fn simple_authorization() {
        let register_request = sample_register_request();
        let controller = get_controller();

        let res = controller.register(register_request).await;
        assert!(res.is_ok());

        let token = res.unwrap();
        let decoded_user = controller.authorize(token).await;
        assert!(decoded_user.is_ok());
    }
}
