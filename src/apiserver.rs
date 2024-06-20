#![allow(non_snake_case)]
#[cfg(not(target_arch = "wasm32"))]
use crate::model::load;
#[cfg(not(target_arch = "wasm32"))]

#[cfg(not(target_arch = "wasm32"))]
use axum::{self,
    response::sse::{Event, Sse},
    Json};
#[cfg(not(target_arch = "wasm32"))]
use futures::stream::{self, Stream};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use dashmap::DashMap;
#[cfg(not(target_arch = "wasm32"))]
use axum::{routing::post, Router};
#[cfg(not(target_arch = "wasm32"))]
use tokio::signal;
#[cfg(not(target_arch = "wasm32"))]
use std::process;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Read;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
#[cfg(not(target_arch = "wasm32"))]
use lazy_static::lazy_static;
#[cfg(not(target_arch = "wasm32"))]
use crate::data::Request;
#[cfg(not(target_arch = "wasm32"))]
use ipc_channel::ipc::{self, IpcOneShotServer, IpcSender, IpcReceiver};
#[cfg(not(target_arch = "wasm32"))]
use tokio_stream::StreamExt as _;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::convert::Infallible;
use crate::data::Message;
#[cfg(not(target_arch = "wasm32"))]
use crate::str_output_stream::IpcReceiverIterator;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use server_fn::codec::{StreamingText, TextStream};


#[cfg(not(target_arch = "wasm32"))]
lazy_static!{
pub static ref WORKER_HUB: Mutex<DashMap<String, Worker>> = Mutex::new(DashMap::<String, Worker>::new());
}

#[server(output = StreamingText)]
pub async fn chat_stream(msg: Vec<Message>,model_id: String, endpoint:String) -> Result<TextStream, ServerFnError> {
    use futures::StreamExt;
    let (tx, rx) = futures::channel::mpsc::unbounded();
    tokio::spawn(async move {
        use reqwest::Client;
        use eventsource_stream::Eventsource;
       
        let mut stream = Client::new().post(endpoint)
            .json(&Request {
                cmd: model_id,
                msg_list: msg,
            })
            .send()
            .await.unwrap()
            .bytes_stream()
            .eventsource();
        while let Some(event) = futures::StreamExt::next(&mut stream).await {
            match event {
                Ok(event) => {
                    let _ = tx.unbounded_send(Ok(event.data));
                }
                Err(_) => {
                    panic!("Error in event stream")
                }
            }
        } 
    });

    Ok(TextStream::new(rx))
}

#[server]
pub async fn chat(msg: Vec<Message>,model_id: String, endpoint:String) -> Result<String, ServerFnError> {
    println!("call chat in server");
    #[cfg(not(target_arch = "wasm32"))]
    let result = {
        
        use reqwest::Client;
        use eventsource_stream::Eventsource;
       
        let mut stream = Client::new().post(endpoint)
            .json(&Request {
                cmd: model_id,
                msg_list: msg,
            })
            .send()
            .await.unwrap()
            .bytes_stream()
            .eventsource();
        let mut content: String = String::new();
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    content.push_str(&event.data);
                }
                Err(_) => {
                    panic!("Error in event stream");
                }
            }
        }
        println!("\nend\n");
        Ok(content)
    };

    #[cfg(target_arch = "wasm32")]
    let result = Ok("Hello from wasm32");
    result
}

#[derive(Deserialize, Serialize, Debug)]
struct WorkerServer {
    pub model_id: String,
    pub temp: f64,
    pub top_p: f64,
}
#[derive(Deserialize, Serialize, Debug)]
struct ServerConfig {
    pub ports: Vec<u32>,
    pub master_addr: String,
    pub worker_servers: Vec<WorkerServer>,
}
#[cfg(not(target_arch = "wasm32"))]
pub struct Worker {
    pub model_id: String,
    pub sender: IpcSender<String>,
    pub receiver: IpcReceiver<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn call_worker(Json(request): Json<Request>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {

    println!("call_worker!!");
    if let Some(worker) = WORKER_HUB.lock().unwrap().get(&request.cmd) {
        
        let req = Request {
            cmd: "chat".to_string(),
            msg_list: request.msg_list,
        };
        let msg = serde_json::json!(req).to_string();
        println!("call model server by {}", msg);
        worker.sender.send(msg).unwrap();
        
        let iter = IpcReceiverIterator::new(Some(request.cmd.clone()),"");
        let stream = stream::iter(iter).map(Ok);
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
                .text("keep-alive-text"),
        )
    } else {
        let stream = stream::iter(IpcReceiverIterator::new(None,"Model can't found!")).map(Ok);
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
              .interval(Duration::from_secs(1))
              .text("keep-alive-text"),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(unix)]
async fn handle_unix_signals() {
    let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal::unix::signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigterm.recv() => {
            println!("Received SIGTERM");
        },
        _ = sigint.recv() => {
            println!("Received SIGINT");
        },
    }
    for mut kv in WORKER_HUB.lock().unwrap().iter_mut() {
        let worker = kv.value_mut();
        let req = Request {
            cmd: "QUIT".to_string(),
            msg_list: Vec::<Message>::new(),
        };
        let msg = serde_json::json!(req).to_string();
        
        let _ = worker.sender.send(msg).is_err_and(|x| { println!("{:?}",x); process::exit(0)});
    }
    process::exit(0);
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn master_server() {
    let mut file = fs::File::open("server.config").expect("Failed to read server.config!");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read server.config to string!");
    let config =
        serde_json::from_str::<ServerConfig>(contents.as_str()).expect("Failed to deseriablize.");
    let program = env::current_exe().expect("Failed to determine the current executable path");
    
    for server in config.worker_servers.iter() {
        let model_id = &server.model_id;
        let (oneShotServ, ipc_name) = IpcOneShotServer::new().expect("Failed to ipc one shot server");
        let _ = Command::new(program.as_os_str())
            .arg("--server")
            .arg("Worker")
            .arg("--model-id")
            .arg(model_id.as_str())
            .arg("--ipc-name")
            .arg(ipc_name.as_str())
            .spawn()
            .expect("Worker server failed to start");
        let (_, sender): (_, IpcSender<String>) = oneShotServ.accept().expect("Failed to accept sender!");
        let (oneShotServ, ipc_name) = IpcOneShotServer::new().unwrap();
        sender.send(ipc_name).expect("Failed to send ipc name");
        let (_, receiver): (_, IpcReceiver<String>) = oneShotServ.accept().expect("Failed to accept receiver!");

        WORKER_HUB.lock().unwrap().insert(
            server.model_id.clone(),
            Worker {
                model_id: model_id.clone(),
                sender: sender,
                receiver: receiver,
            },
        );
    }
    #[cfg(unix)]
    let rt = tokio::runtime::Runtime::new().expect("Create runtime failed!");
    #[cfg(unix)]
    rt.spawn(handle_unix_signals());
    
    
    let app = Router::new().route("/chat", post(call_worker));
    let listener = tokio::net::TcpListener::bind(config.master_addr.clone())
        .await
        .unwrap();
    println!("listenning in {}",config.master_addr);
    axum::serve(listener, app).await.unwrap();
}
