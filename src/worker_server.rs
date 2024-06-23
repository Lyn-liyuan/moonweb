use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use crate::data::Request;
use crate::model::load;
use std::process;

pub async fn worker_server(ipc_name:String, model_id: String, temp: f64, top_p: f64) {
    let mut pipeline = load(&model_id, temp, top_p).expect("Failed to load model!");
    let (client_sender, receiver): (IpcSender<String>, IpcReceiver<String>) = ipc::channel().unwrap();
    let connector = IpcSender::connect(ipc_name.clone()).expect(format!("Failed to connect {}",ipc_name).as_str());
    connector.send(client_sender).expect("Failed to send client sender");
    let (sender, client_receiver): (IpcSender<String>, IpcReceiver<String>) = ipc::channel().unwrap();
    let client_name = receiver.recv().expect("Failed to recv!");
    let connector = IpcSender::connect(client_name.clone()).expect(format!("Failed to connect client: {}",client_name).as_str());
    connector.send(client_receiver).expect("Failed to send client receive");

    println!("model {} server start!", model_id);
    loop {
        let request: String = receiver.recv().expect("Failed to recv!");
        if let Ok(req) = serde_json::from_str::<Request>(request.as_str()) {
            if req.cmd.eq("QUIT") {
                break;
            }
            let msg_list = req.msg_list;
            let history =
                pipeline.messages_chat_template(&msg_list, "You are hulpful AI assistant.");
            let _ = pipeline.run(&sender,history.as_str(), 1000usize).unwrap();    
        }
    }
    process::exit(0);
}