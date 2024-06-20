use core::str;

use crate::data::Message;
use crate::llama;
use anyhow::{Error, Result};
use crate::str_output_stream::OutputStream;


pub trait TextGenModel {
    fn run(&mut self, output:&dyn OutputStream,prompt: &str, sample_len: usize) -> Result<(), Error>;
    fn messages_chat_template(&self, msg_list: &Vec<Message>, system_prompt: &str) -> String;
}

pub fn load(model_id: &str, temp: f64, top_p: f64) -> Option<Box<dyn TextGenModel>> {
    match model_id {
        "meta-llama/Meta-Llama-3-8B-Instruct" => Some(Box::new(llama::load_model(model_id, temp, top_p))),
        "microsoft/Phi-3-medium-4k-instruct" => Some(Box::new(crate::phi3::load())),
        _ => None,
    }
}
