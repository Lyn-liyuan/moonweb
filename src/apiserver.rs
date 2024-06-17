#![allow(non_snake_case)]
#[cfg(not(target_arch = "wasm32"))]
use crate::model::load;
#[cfg(not(target_arch = "wasm32"))]
use axum;
#[cfg(not(target_arch = "wasm32"))]
use axum::http::StatusCode;
#[cfg(not(target_arch = "wasm32"))]
use axum::Json;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use dashmap::DashMap;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{sleep, Duration};
#[cfg(not(target_arch = "wasm32"))]
use zeromq::*;
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
use reqwest::Client;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::Message;


#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    pub static ref WORKER_HUB: Arc<DashMap<String, Box<Worker>>> = Arc::new(DashMap::<String, Box<Worker>>::new());
}

#[server]
pub async fn chat(msg: Vec<Message>,model_id: String, endpoint:String) -> Result<String, ServerFnError> {
    println!("call chat in server");
    #[cfg(not(target_arch = "wasm32"))]
    let result = {
        use reqwest::StatusCode;
        let client = Client::new();
        let response = client
            .post("http://127.0.0.1:3081/chat")
            .json(&Request {
                cmd: model_id,
                msg_list: msg,
            })
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            Ok(response.text().await?)
        } else {
            Ok("I can't do it!".to_string())
        }
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
    pub port: u32,
    pub socket: Box<ReqSocket>,
}
#[cfg(not(target_arch = "wasm32"))]
pub fn zmq_message_to_string(message: &ZmqMessage) -> Result<String, std::string::FromUtf8Error> {
    let mut result = String::new();
    for frame in message.iter() {
        let s = String::from_utf8(frame.to_vec())?;
        result.push_str(&s);
    }
    Ok(result)
}
#[cfg(not(target_arch = "wasm32"))]
pub async fn call_worker(Json(request): Json<Request>) -> (StatusCode, String) {
    println!("call_worker!!");
    if let Some(ref mut worker) = WORKER_HUB.clone().get_mut(&request.cmd) {
        let socket = &mut worker.socket;
        let req = Request {
            cmd: "chat".to_string(),
            msg_list: request.msg_list,
        };
        let msg = serde_json::json!(req).to_string();
        println!("call model server by {}", msg);
        socket.send(msg.into()).await.unwrap();
        let repl = socket.recv().await.unwrap();
        let resp = zmq_message_to_string(&repl).unwrap();
        (StatusCode::OK, resp)
    } else {
        (
            StatusCode::BAD_REQUEST,
            format!("Not Found model:{}", request.cmd),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn worker_server(model_id: String, port: u32, temp: f64, top_p: f64) {
    let mut pipeline = load(&model_id, temp, top_p).expect("Failed to load model!");
    let mut socket = zeromq::RepSocket::new();
    socket
        .bind(format!("tcp://127.0.0.1:{}", port).as_str())
        .await
        .unwrap();
    println!("model {} server start!", model_id);
    loop {
        let request: String = socket.recv().await.unwrap().try_into().unwrap();
        if let Ok(req) = serde_json::from_str::<Request>(request.as_str()) {
            if req.cmd.eq("QUIT") {
                break;
            }
            let msg_list = req.msg_list;
            let history =
                pipeline.messages_chat_template(&msg_list, "You are hulpful AI assistant.");
            let response = pipeline.run(history.as_str(), 1000usize).unwrap();
            socket.send(response.into()).await.unwrap();
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
    for mut kv in WORKER_HUB.clone().iter_mut() {
        let worker = kv.value_mut();
        let req = Request {
            cmd: "QUIT".to_string(),
            msg_list: Vec::<Message>::new(),
        };
        let msg = serde_json::json!(req).to_string();
        println!("call model server by {}", msg);
        worker.socket.send(msg.into()).await.unwrap();
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
    
    for (index, server) in config.worker_servers.iter().enumerate() {
        let model_id = &server.model_id;
        let port = config.ports.get(index).unwrap().clone();
        let _ = Command::new(program.as_os_str())
            .arg("--server")
            .arg("Worker")
            .arg("--model-id")
            .arg(model_id.as_str())
            .arg("--port")
            .arg(port.to_string().as_str())
            .spawn()
            .expect("Worker server failed to start");
        sleep(Duration::from_secs(1)).await;
        let mut new_socket = Box::new(ReqSocket::new());
        new_socket
            .connect(format!("tcp://127.0.0.1:{}", port).as_str())
            .await
            .expect("Failed to connect");
        WORKER_HUB.clone().insert(
            server.model_id.clone(),
            Box::new(Worker {
                model_id: model_id.clone(),
                port: port,
                socket: new_socket,
            }),
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
