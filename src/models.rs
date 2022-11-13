use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Game {
    pub questions: Vec<Question>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Question {
    pub question: String,
    pub options: [String; 4],
    pub answer_idx: OptionIndex,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Options([String; 4]);

impl Options {
    fn _get(&self, index: OptionIndex) -> &str {
        let index = index as usize;
        &self.0[index]
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum OptionIndex {
    One,
    Two,
    Three,
    Four,
}

pub type GameStartSignalReceiver = Arc<tokio::sync::Mutex<UnboundedReceiver<()>>>;

pub struct Client {
    pub id: usize,
    pub tx: Tx,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    TimeTillGame {
        time: u64,
    },
    Question {
        question: String,
        options: [String; 4],
    },
    Answer {
        status: AnswerStatus,
        answer_idx: OptionIndex,
    },
    NoGame,
    GameEnd,
    GameStart,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnswerStatus {
    Correct,
    Incorrect,
    NoAnswer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Answer { answer_idx: OptionIndex },
}
