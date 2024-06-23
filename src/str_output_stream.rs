use anyhow::{Error, Result};
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

