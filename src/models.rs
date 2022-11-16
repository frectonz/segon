use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Game {
    pub questions: Vec<Question>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Question {
    pub question: String,
    pub options: [String; 4],
    pub answer_idx: OptionIndex,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum OptionIndex {
    One,
    Two,
    Three,
    Four,
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
    GameEnd {
        score: u32,
    },
    GameStart,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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
