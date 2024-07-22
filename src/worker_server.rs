
use crate::data::{Request,Role,Message};
use crate::model::load;
use crate::ipc::accept;
use std::process;

pub async fn worker_server(ipc_name:String, model_id: String, temp: f64, top_p: f64) {
    
    let (receiver, sender) = accept(ipc_name);
    
    let mut pipeline = load(&model_id, temp, top_p).expect("Failed to load model!");
    println!("model {} server start!", model_id);
    loop {
        let request: String = receiver.recv().expect("Failed to recv!");
        if let Ok(req) = serde_json::from_str::<Request>(request.as_str()) {
            if req.cmd.eq("QUIT") {
                break;
            }
            let msg_list: Vec<Message> = req.msg_list.into_iter().filter(|msg|msg.role!=Role::Admin).collect();
            let history =
                pipeline.messages_chat_template(&msg_list, req.system_prompt.as_str());
            let _ = pipeline.run(&sender,history.as_str(), 1000usize).unwrap();    
        }
    }
    process::exit(0);
}

