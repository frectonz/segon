use crate::ports::{Claims, TokenGenerator};

pub struct SimpleTokenGenerator;

impl TokenGenerator for SimpleTokenGenerator {
    type Error = std::convert::Infallible;

    fn generate<'a>(&'a self, username: &str) -> Result<String, Self::Error> {
        Ok(username.into())
    }

    fn get_claims<'a>(&'a self, token: String) -> Option<Claims> {
        Some(Claims {
            iat: 1,
            exp: 10,
            sub: token,
        })
    }
}
