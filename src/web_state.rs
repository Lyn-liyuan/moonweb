use crate::data::Message;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use js_sys::Date;
use web_sys::{window, Storage};

#[derive(Props, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub mode_id: String,
    pub system_prompt: String,
    pub history: Option<Vec<Message>>,
}

pub struct TempSession {
    pub id: String,
    pub name: String,
    pub mode_id: String,
    pub system_prompt: String,
}

impl TempSession {
    pub fn new(session: &Session) -> TempSession {
        TempSession {
            id: session.id.clone(),
            name: session.name.clone(),
            mode_id: session.mode_id.clone(),
            system_prompt: session.system_prompt.clone(),
        }
    }
}

pub struct Store {
    local_storage: Storage,
    session_list: Vec<String>,
}

impl Store {
    pub fn new() -> Option<Store> {
        let window = window()?;
        if let Ok(Some(local_storage)) = window.local_storage() {
            let session_list = if let Ok(Some(value)) = local_storage.get_item("session_list") {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(value.as_str()) {
                    list
                } else {
                    local_storage.set_item("session_list", "[]").unwrap();
                    Vec::<String>::new()
                }
            } else {
                local_storage.set_item("session_list", "[]").unwrap();
                let list = Vec::<String>::new();
                list
            };
            let store = Store {
                local_storage,
                session_list,
            };
            Some(store)
        } else {
            None
        }
    }

    pub fn new_session(&mut self) -> Session {
        let time = Date::new_0();
        let id = format!("id_{}_", Date::now());
        let session = Session {
            id: id.clone(),
            name: format!("{}/{}/{} {}:{}", time.get_full_year()-2000,time.get_month()+1,time.get_date(),time.get_hours(),time.get_minutes()),
            system_prompt: "".to_string(),
            mode_id: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
            history: None,
        };
        session
    }

    pub fn save_session(&mut self, session: &Session) {
        let id = session.id.clone();
        let exist = if let Ok(None) = self.local_storage.get_item(id.as_str()) {
            false
        } else {
            true
        };
        let binding = serde_json::json!(session).to_string();
        let value = binding.as_str();
        self.local_storage.set_item(id.as_str(), value).unwrap();
        if !exist {
            self.session_list.push(id.clone());
            self.local_storage
                .set_item(
                    "session_list",
                    serde_json::json!(self.session_list).to_string().as_str(),
                )
                .expect("Failed to save list!");
        }
    }

    pub fn fetch_all_session(&mut self) -> Vec<Session> {
        let list = self.session_list.clone();
        list.iter()
            .map(|id| {
                let session = if let Ok(Some(value)) = self.local_storage.get_item(id) {
                    if let Ok(sess) = serde_json::from_str::<Session>(value.as_str()) {
                        sess
                    } else {
                        self.new_session()
                    }
                } else {
                    self.new_session()
                };
                session
            })
            .collect()
    }

    pub fn get_session(&self, id: &str) -> Option<Session> {
        let session = if let Ok(Some(value)) = self.local_storage.get_item(id) {
            if let Ok(sess) = serde_json::from_str::<Session>(value.as_str()) {
                Some(sess)
            } else {
                None
            }
        } else {
            None
        };
        session
    }

    pub fn remove_session(&mut self, id: &str) -> Result<(), &'static str> {
        if let Ok(()) = self.local_storage.delete(id) {
            self.session_list.retain(|sid| sid != id);
            self.local_storage
                .set_item(
                    "session_list",
                    serde_json::json!(self.session_list).to_string().as_str(),
                )
                .expect("Failed to save list!");
            Ok(())
        } else {
            Err("delete failed!")
        }
    }
}
