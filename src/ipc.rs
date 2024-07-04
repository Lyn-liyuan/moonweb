use anyhow::{Error, Result};


use ipc_channel::ipc::{self, IpcSender, IpcReceiver};

pub fn accept(ipc_name: String) -> (IpcReceiver<String>, IpcSender<String>) {
    let (client_sender, receiver): (IpcSender<String>, IpcReceiver<String>) = ipc::channel().unwrap();
    let connector = IpcSender::connect(ipc_name.clone()).expect(format!("Failed to connect {}",ipc_name).as_str());
    connector.send(client_sender).expect("Failed to send client sender");
    let (sender, client_receiver): (IpcSender<String>, IpcReceiver<String>) = ipc::channel().unwrap();
    let client_name = receiver.recv().expect("Failed to recv!");
    let connector = IpcSender::connect(client_name.clone()).expect(format!("Failed to connect client: {}",client_name).as_str());
    connector.send(client_receiver).expect("Failed to send client receive");
    (receiver, sender)
}

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


