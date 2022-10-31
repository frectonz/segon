use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use warp::ws::Message;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password: String,
}

pub type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<usize, Tx>>>;
pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Game {
    pub questions: Vec<Question>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Question {
    pub question: String,
    pub options: OptionIndex,
    pub answer_idx: OptionIndex,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Options([String; 4]);

impl Options {
    fn get(&self, index: OptionIndex) -> &str {
        let index = index as usize;
        &self.0[index]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum OptionIndex {
    One = 1,
    Two,
    Three,
    Four,
}

pub type GameStartSignalReceiver = Arc<tokio::sync::Mutex<UnboundedReceiver<String>>>;
