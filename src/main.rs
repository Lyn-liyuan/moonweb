#![allow(non_snake_case, unused)]

use clap::*;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use moonweb::web::app;
use std::str::FromStr;

#[cfg(not(target_arch = "wasm32"))]
use moonweb::master_server::master_server;
#[cfg(not(target_arch = "wasm32"))]
use moonweb::worker_server::worker_server;

// Urls are relative to your Cargo.toml file
const _TAILWIND_URL: &str = manganis::mg!(file("public/tailwind.css"));
#[derive(Debug)]
enum ServerNode {
    Master,
    Worker,
    Web,
}

impl FromStr for ServerNode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "master" => Ok(ServerNode::Master),
            "worker" => Ok(ServerNode::Worker),
            "web" => Ok(ServerNode::Web),
            _ => Err(format!("'{}' is not a valid ServerNode", s)),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    server: Option<ServerNode>,

    #[clap(short, long)]
    ipc_name: Option<String>,

    #[clap(short, long)]
    model_id: Option<String>,

    #[clap(short='h', long)]
    temp: Option<f64>,

    #[clap(short='t', long)]
    top_p: Option<f64>,

    #[clap(short='e', long)]
    master_port: Option<u32>,
    
}

fn main() {
    let args = Args::parse();
    let server_type = args.server.unwrap_or_else(||ServerNode::Web);
    
    match server_type {
        ServerNode::Web => {
            dioxus_logger::init(Level::INFO).expect("logger failed to init");

            launch(app);
        }
        ServerNode::Master => {
            #[cfg(not(target_arch = "wasm32"))] 
            {
                let runtime = tokio::runtime::Runtime::new().expect("Create runtime failed!");
                runtime.block_on(master_server());
            }
            
        }
        ServerNode::Worker => {
            #[cfg(not(target_arch = "wasm32"))] 
            {
            let model_id = args
                .model_id
                .unwrap_or_else(|| "meta-llama/Meta-Llama-3-8B-Instruct".into());
            let temp = args.temp.unwrap_or_else(|| 0.6f64);
            let top_p = args.top_p.unwrap_or_else(|| 0.9f64);
            let ipc_name = args.ipc_name.unwrap();
            let runtime = tokio::runtime::Runtime::new().expect("Create runtime failed!");
            runtime.block_on(worker_server(
                ipc_name,
                model_id.clone(),
                temp,
                top_p,
            ));
        }
        }
    }
}
