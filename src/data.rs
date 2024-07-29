use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Role {
    Robot,
    User,
    Administrator,
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "User" => Ok(Role::User),
            "Robot" => Ok(Role::Robot),
            "Administrator" => Ok(Role::Administrator),
            _ => Err(format!("'{}' is not a valid value for Role", s)),
        }
    }
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
    pub system_prompt:String,
    pub msg_list:Vec<Message>,
}

#[derive(Props, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SelectOption {
    pub text:String,
    pub selected: bool,
    pub value: String,
}

#[derive(Props, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub role: Role,
    pub token: String,
}

#[derive(Props, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub auth_key: String,
    pub expire: String,
}

#[derive(Props, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebUser {
    pub role: Role,
    pub auth_key: Option<String>,
    pub expire: Option<String>,
}

impl WebUser {
    pub fn new() ->Self {
        WebUser {
            role: Role::User,
            auth_key: None,
            expire: None,
        }
    }
    pub fn make(role:Role,key:String,expire: String) -> Self {
        WebUser {
            role: role,
            auth_key: Some(key),
            expire: Some(expire),
        }
    }
}


