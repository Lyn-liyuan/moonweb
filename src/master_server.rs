use crate::data::Request;
use axum::{
    self,
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use std::convert::Infallible;
use std::env;
use std::fs;
use std::io::Read;
use std::process::Command;
use tokio::signal;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
use crate::data::Message;
use axum::{routing::post, Router};
use dashmap::DashMap;
use std::process;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Duration;

lazy_static! {
    pub static ref WORKER_HUB: DashMap<String, Worker> = DashMap::<String, Worker>::new();
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
    pub sender: Sender<(Sender<String>, String)>,
}

async fn proxy(
    sender: IpcSender<String>,
    receiver: IpcReceiver<String>,
    mut rx: Receiver<(Sender<String>, String)>,
) {
    loop {
        if let Some((response_tx, request_data)) = rx.recv().await {
            sender
                .send(request_data)
                .expect("Failed to send request to worker process!");

            loop {
                let response = receiver.recv().expect("Failed to receive from worker");
                if response == "<|endoftext|>" {
                    break;
                }
                if response_tx.send(response).await.is_err() {
                    break;
                }
            }
        };
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn call_worker(
    Json(request): Json<Request>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("call_worker!! {}", request.cmd);
    let model_id = request.cmd;

    let mut receiver = if let Some(worker) = WORKER_HUB.get(&model_id) {
        let (response_tx, response_rx) = mpsc::channel::<String>(1);
        let req = Request {
            cmd: "chat".to_string(),
            msg_list: request.msg_list,
        };
        let msg = serde_json::json!(req).to_string();
        worker
            .sender
            .send((response_tx, msg))
            .await
            .expect("Failed to send to worker");
        Some(response_rx)
    } else {
        None
    };
    use tokio_stream::StreamExt as _;

    let stream = async_stream::stream! {
        match receiver {
                Some(ref mut rx)=> loop {
                     let msg = match rx.recv().await {
                        Some(text) => Some(text),
                        None => {
                            break;
                        }
                     };
                     yield msg;
                },
                None => loop {
                    println!("worker is None!!!!");
                    yield None;
                    break;
                }
        }
    }
    .map(|msg| match msg {
        Some(text) => Event::default().data(text),
        None => Event::default().data("[DONE]"),
    })
    .map(Ok);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(2))
            .text("keep-alive-text"),
    )
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
    for mut kv in WORKER_HUB.iter_mut() {
        let worker = kv.value_mut();
        let req = Request {
            cmd: "QUIT".to_string(),
            msg_list: Vec::<Message>::new(),
        };
        let msg = serde_json::json!(req).to_string();
        let (response_tx, _) = mpsc::channel::<String>(1);

        let _ = worker
            .sender
            .send((response_tx, msg))
            .await
            .is_err_and(|x| {
                println!("{:?}", x);
                process::exit(0)
            });
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
        let (one_shot_serv, ipc_name) =
            IpcOneShotServer::new().expect("Failed to ipc one shot server");
        let _ = Command::new(program.as_os_str())
            .arg("--server")
            .arg("Worker")
            .arg("--model-id")
            .arg(model_id.as_str())
            .arg("--ipc-name")
            .arg(ipc_name.as_str())
            .spawn()
            .expect("Worker server failed to start");
        let (_, sender): (_, IpcSender<String>) =
            one_shot_serv.accept().expect("Failed to accept sender!");
        let (one_shot_serv, ipc_name) = IpcOneShotServer::new().unwrap();
        sender.send(ipc_name).expect("Failed to send ipc name");
        let (_, receiver): (_, IpcReceiver<String>) =
            one_shot_serv.accept().expect("Failed to accept receiver!");
        let (tx, rx) = mpsc::channel::<(Sender<String>, String)>(100);
        WORKER_HUB.insert(
            server.model_id.clone(),
            Worker {
                model_id: model_id.clone(),
                sender: tx,
            },
        );
        tokio::spawn(proxy(sender, receiver, rx));
    }
    #[cfg(unix)]
    let rt = tokio::runtime::Runtime::new().expect("Create runtime failed!");
    #[cfg(unix)]
    rt.spawn(handle_unix_signals());

    let app = Router::new().route("/api/chat", post(call_worker));
    let listener = tokio::net::TcpListener::bind(config.master_addr.clone())
        .await
        .unwrap();
    println!("listenning in {}", config.master_addr);
    axum::serve(listener, app).await.unwrap();
}
