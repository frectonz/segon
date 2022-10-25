use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[async_trait]
pub trait TokenGenerator<Token: Display + Serialize + for<'a> Deserialize<'a>> {
    fn generate<'a>(&'a self, username: &str) -> Token;
    fn get_username<'a>(&'a self, token: Token) -> Option<String>;
}
