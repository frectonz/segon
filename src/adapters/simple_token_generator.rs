use crate::ports::TokenGenerator;

pub struct SimpleTokenGenerator;

impl TokenGenerator<String> for SimpleTokenGenerator {
    fn generate< 'a>(& 'a self, username: &str) -> String {
        username.into()
    }

    fn get_username<'a>(& 'a self, token: String) -> Option<String> {
        Some(token)
    }
}
