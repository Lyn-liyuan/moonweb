#[cfg(not(target_arch = "wasm32"))]
use crate::apiserver::WORKER_HUB;
use anyhow::{Error, Result};
use axum::response::sse::Event;
use ipc_channel::ipc::IpcSender;

pub trait OutputStream {
    fn write(&self, text: String) -> Result<(), Error>;
    fn end(&self) -> Result<(), Error>;
}

impl OutputStream for IpcSender<String> {
    fn write(&self, text: String) -> Result<(), Error> {
        self.send(text)?;
        Ok(())
    }

    fn end(&self) -> Result<(), Error> {
        self.send("<|endoftext|>".to_string())?;
        Ok(())
    }
}

pub struct IpcReceiverIterator {
    model_id: Option<String>,
    error_info: Vec<String>,
}

impl IpcReceiverIterator {
    pub fn new(model_id: Option<String>, error: &str) -> Self {
        IpcReceiverIterator {
            model_id: model_id,
            error_info: vec![error.to_string()],
        }
    }
}

impl Iterator for IpcReceiverIterator {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(model_id) = &self.model_id {
            if let Some(worker) = {WORKER_HUB.lock().unwrap().get(model_id)} {
                match worker.receiver.recv() {
                    Ok(msg) => {
                        if msg == "<|endoftext|>" {
                            None
                        } else {
                            Some(Event::default().data(msg.as_str()))
                        }
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            if let Some(error) = self.error_info.iter().next() {
                Some(Event::default().data(error.as_str()))
            } else {
                None
            }
        }
    }
}
