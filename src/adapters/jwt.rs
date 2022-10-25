use crate::ports::TokenGenerator;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const SECRET: &str = "secret";

pub struct Jwt;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: u64, // Optional. Issued at (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
}

impl TokenGenerator<String> for Jwt {
    fn generate<'a>(&'a self, username: &str) -> String {
        let claims = Claims {
            exp: 24 * 60 * 60,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sub: username.into(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .unwrap();

        token
    }

    fn get_username<'a>(&'a self, token: String) -> Option<String> {
        decode::<Claims>(
            &token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &Validation::default(),
        )
        .ok()
        .map(|data| data.claims.sub)
    }
}
