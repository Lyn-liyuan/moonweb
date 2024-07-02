use moonweb::ipc::{accept,OutputStream};
use moonweb::data::Request;
use clap::*;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    server: Option<String>,

    #[clap(short, long)]
    ipc_name: Option<String>,

    #[clap(short, long)]
    model_id: Option<String>,

    #[clap(short='h', long)]
    temp: Option<f64>,

    #[clap(short='t', long)]
    top_p: Option<f64>,

}

fn main() {
    let args = Args::parse();
    let ipc_name = args.ipc_name.unwrap();
    let model_id = args.model_id.unwrap();
    let (receiver,sender) = accept(ipc_name);
    println!("{} server start!",model_id);
    loop {
        let msg = receiver.recv().unwrap();
            if let Ok(req) = serde_json::from_str::<Request>(msg.as_str()) {
                if req.cmd.eq("QUIT") {
                    break;
                }
            let response = format!("{} recv {:?}",model_id,req.msg_list);
            for char in response.chars() {
                sender.write(format!("{}",char)).unwrap();
            }
            sender.end().unwrap();
        }
    }
    
}
