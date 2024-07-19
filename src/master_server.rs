use crate::data::Message;
use crate::data::Request;
use crate::master_state::{
    get_master_addr, get_program, get_servers, get_working_servers, new_working_server,
    remove_working_server,
};
use axum::{
    self,
    response::sse::{Event, Sse},
    extract::DefaultBodyLimit,
    Json,
};
use axum::{
    routing::{get, post},
    Router,
};

use tower_http::services::{ServeDir, ServeFile};
use dashmap::DashMap;
use futures::stream::Stream;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use lazy_static::lazy_static;
use std::convert::Infallible;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use tokio::signal;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Duration;

lazy_static! {
    static ref WORKER_HUB: DashMap<String, Worker> = DashMap::<String, Worker>::new();
}
pub struct Worker {
    pub model_id: String,
    pub sender: Sender<(Option<Sender<String>>, Request)>,
}

async fn modal_actor(
    sender: IpcSender<String>,
    receiver: IpcReceiver<String>,
    mut rx: Receiver<(Option<Sender<String>>, Request)>,
) {
    loop {
        if let Some((response_tx, request_data)) = rx.recv().await {
            let data = serde_json::json!(request_data).to_string();
            sender
                .send(data)
                .expect("Failed to send request to worker process!");
            if request_data.cmd == "QUIT" {
                break;
            }
            loop {
                if let Ok(response) = receiver.recv() {
                    if response == "<|endoftext|>" {
                        break;
                    }
                    if response_tx.clone().unwrap().send(response).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        };
    }
}

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
        
        worker
            .sender
            .send((Some(response_tx), req))
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

        let _ = worker
            .sender
            .send((None, req))
            .await
            .is_err_and(|x| {
                println!("{:?}", x);
                process::exit(0)
            });
    }
    process::exit(0);
}

pub async fn master_server() {
    
    for server in get_working_servers().await.iter() {
        let model_id = &server.model_id;
        let program = get_program(server);
        launch_worker(&program, model_id);
    }
    
    #[cfg(unix)]
    let rt = tokio::runtime::Runtime::new().expect("Create runtime failed!");
    #[cfg(unix)]
    rt.spawn(handle_unix_signals());

    let serve_dir = ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html"));

    let app = Router::new()
        .route("/api/chat", post(call_worker))
        .route("/api/load", post(call_command))
        .route("/api/unload", post(call_command))
        .route("/api/models", get(modal_list))
        .layer(DefaultBodyLimit::disable())
        .nest_service("/", serve_dir.clone())
        .fallback_service(serve_dir);
    let addr = get_master_addr().await;
    let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();
    println!("listenning in {}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn launch_worker(program: &PathBuf, model_id: &String) {
    let (one_shot_serv, ipc_name) = IpcOneShotServer::new().expect("Failed to ipc one shot server");
    let e = Command::new(program.as_os_str())
        .arg("--server")
        .arg("Worker")
        .arg("--model-id")
        .arg(model_id.as_str())
        .arg("--ipc-name")
        .arg(ipc_name.as_str())
        .spawn();
    if e.is_err() {
        println!("Worker server {} failed to start",model_id);
        return;
    }
       
    let (_, sender): (_, IpcSender<String>) =
        one_shot_serv.accept().expect("Failed to accept sender!");
    let (one_shot_serv, ipc_name) = IpcOneShotServer::new().unwrap();
    sender.send(ipc_name).expect("Failed to send ipc name");
    let (_, receiver): (_, IpcReceiver<String>) =
        one_shot_serv.accept().expect("Failed to accept receiver!");
    let (tx, rx) = mpsc::channel::<(Option<Sender<String>>, Request)>(1);
    WORKER_HUB.insert(
        model_id.clone(),
        Worker {
            model_id: model_id.clone(),
            sender: tx,
        },
    );
    tokio::spawn(modal_actor(sender, receiver, rx));
}

pub async fn modal_list() -> Json<Vec<String>> {
    let list: Vec<String> = get_working_servers()
        .await
        .iter()
        .map(|serv| serv.model_id.clone())
        .collect();
    Json::from(list)
}

pub async fn call_command(cmd: String) -> String {
    let commands: Vec<&str> = cmd
        .split(|c: char| c.is_whitespace())
        .filter(|&s| !s.is_empty())
        .collect();
    if commands.len() > 1 {
        match commands[0] {
            "/load" => {
                let model_id = commands[1].to_string();

                match get_working_servers()
                    .await
                    .iter()
                    .find(|ser| ser.model_id == model_id)
                {
                    None => {
                        if let Some(server) = get_servers()
                            .await
                            .iter()
                            .find(|ser| ser.model_id == model_id)
                        {
                            let program = get_program(server);
                            launch_worker(&program, &model_id);
                            new_working_server(server.clone()).await;
                            format!("{} server start!", model_id)
                        } else {
                            format!("{} is not exist!", model_id)
                        }
                    }
                    Some(_) => format!("{} server is runing!", model_id),
                }
            },
            "/unload" => {
                let model_id = commands[1].to_string();
                if let Some((_,server)) = WORKER_HUB.remove(model_id.as_str()) {
                    let req = Request {
                        cmd: "QUIT".to_string(),
                        msg_list: Vec::<Message>::new(),
                    };
                    server.sender.send((None,req)).await.unwrap();
                    remove_working_server(model_id.as_str()).await;
                    format!("{} server stop!", model_id)
                } else {
                    format!("{} server is not runing", model_id)
                }
            },
            _ => {
                format!("Command {} is not exist", commands[0])
            }
        }
    } else {
        format!("{} is error command!", cmd)
    }
}


