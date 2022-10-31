use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub password: String,
}

pub type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<usize, Tx>>>;
pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
