use clap::*;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use pyo3::prelude::*;
use moonweb::ipc::accept;


#[pyclass]
struct IpcChannel {
    sender: IpcSender<String>,
    receiver: IpcReceiver<String>,
}

#[pymethods]
impl IpcChannel {
    #[new]
    fn new(ipc_name: String) -> Self {
        let (receiver, sender) = accept(ipc_name);
        IpcChannel { sender, receiver }
    }

    fn send(&self, msg: &str) -> PyResult<()> {
        self.sender
            .send(msg.to_string())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn recv(&self) -> PyResult<String> {
        self.receiver
            .recv()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

#[pymodule]
fn moonipc(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<IpcChannel>()?;
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long)]
    server: Option<String>,
    #[clap(short, long)]
    ipc_name: Option<String>,
    #[clap(short, long)]
    model_id: Option<String>,

    #[clap(short = 'h', long)]
    temp: Option<f64>,

    #[clap(short = 't', long)]
    top_p: Option<f64>,
}

fn main() {
    let args = Args::parse();
    let ipc_name = args.ipc_name.unwrap();
    let model_id = args.model_id.unwrap();

    let code = include_str!("llavanext.py");

    pyo3::append_to_inittab!(moonipc);
    pyo3::prepare_freethreaded_python();
    let args = (ipc_name.as_str(), model_id.as_str());
    Python::with_gil(|py| {
        let activators = PyModule::from_code_bound(py, code, "llavanext.py", "llavanext").unwrap();
        activators.getattr("run").unwrap().call1(args).unwrap();
    });
}
