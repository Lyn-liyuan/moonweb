use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Role {
    Robot,
    User,
}

#[derive(Props, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: usize,
    pub role: Role,
    pub content: String,
    pub img: Option<String>,
    pub loading: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub cmd:String,
    pub msg_list:Vec<Message>,
}

