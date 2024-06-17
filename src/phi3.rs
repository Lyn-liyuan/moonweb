use anyhow::{Error, Result};

use crate::token_output_stream::TokenOutputStream;

use candle_transformers::models::phi3::{Config as Phi3Config, Model as Phi3};


use candle_core::{DType, Device, IndexOp, Tensor};
use candle_core::utils::cuda_is_available;
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use crate::model::TextGenModel;
use crate::data::{Message,Role};


pub struct TextGeneration {
    model: Phi3,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: Phi3,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
    ) -> Self {
        let logits_processor = LogitsProcessor::new(seed, temp, top_p);
        Self {
            model,
            tokenizer: TokenOutputStream::new(tokenizer),
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
        }
    }
}

impl TextGenModel for TextGeneration {
    

    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<String, Error> {
        use std::io::Write;
        println!("starting the inference loop");
        self.model.clear_kv_cache();
        self.tokenizer.clear();
        let tokens = self.tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(Error::msg)?;
        if tokens.is_empty() {
            anyhow::bail!("Empty prompts are not supported in the phi model.")
        }

        let mut tokens = tokens.get_ids().to_vec();
        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("<|endoftext|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the endoftext token"),
        };
        print!("{prompt}");
        std::io::stdout().flush()?;
        let start_gen = std::time::Instant::now();
        let mut pos = 0;
        //let mut content = String::new();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &self.device).expect("create input tensor failed!").unsqueeze(0).expect("unsqueeze failed!");
            let logits =self.model.forward(&input, pos).expect(format!("model forward failed at {}",index).as_str()).i((.., 0, ..)).expect("i failed!");
            let logits = logits.squeeze(0).expect("logits.squeeze failed!").to_dtype(DType::F32).expect("to dtype failed!");
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = self.logits_processor.sample(&logits).expect("logits processor sample failed！");
            tokens.push(next_token);
            generated_tokens += 1;
            if next_token == eos_token {
                break;
            }
            self.tokenizer.put_token(next_token);
            // if let Some(t) = self.tokenizer.next_token(next_token).expect("tokenizer netx_token failed!") {
            //     print!("{t}");
            //     content.push_str(t.as_str());
            //     std::io::stdout().flush()?;
            // }
            pos += context_size;
        }
        let content = self.tokenizer.decode_all().expect("decode text failed!");
        let dt = start_gen.elapsed();
        println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            generated_tokens as f64 / dt.as_secs_f64(),
        );
        Ok(content)
    }
    
    fn messages_chat_template(&self,msg_list: &Vec<Message>,system_prompt:&str)->String {
        let mut history = String::new();
        history.push_str(system_prompt);
        history.push_str("\n");
        for msg in msg_list {
            if msg.role == Role::User {
                history.push_str("<|user|>\n");
            } else {
                history.push_str("<|assistant|>\n");
            }
            history.push_str(msg.content.as_str());
            history.push_str("<|end|>\n");
        }
        history.push_str("<|assistant|>\n");
        history
    }
}

fn hub_load_safetensors(
    repo: &hf_hub::api::sync::ApiRepo,
    json_file: &str,
) -> Result<Vec<std::path::PathBuf>> {
    let json_file = repo.get(json_file).map_err(candle_core::Error::wrap)?;
    let json_file = std::fs::File::open(json_file)?;
    let json: serde_json::Value =
        serde_json::from_reader(&json_file).map_err(candle_core::Error::wrap)?;
    let weight_map = match json.get("weight_map") {
        None => anyhow::bail!("no weight map in {json_file:?}"),
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => anyhow::bail!("weight map in {json_file:?} is not a map"),
    };
    let mut safetensors_files = std::collections::HashSet::new();
    for value in weight_map.values() {
        if let Some(file) = value.as_str() {
            safetensors_files.insert(file.to_string());
        }
    }
    let safetensors_files = safetensors_files
        .iter()
        .map(|v| repo.get(v).map_err(Error::new))
        .collect::<Result<Vec<_>>>()?;
    Ok(safetensors_files)
}

pub fn load() -> impl TextGenModel {
   let model_id = String::from("microsoft/Phi-3-medium-4k-instruct");
   let revision = String::from("main");
   let api = Api::new().expect("hf_hub api load failed!");
   let repo = api.repo(Repo::with_revision(model_id, RepoType::Model, revision));
   let tokenizer_filename = repo.get("tokenizer.json").expect("load tokenizer.json failed !");
   let filenames = hub_load_safetensors(
                    &repo,
         "model.safetensors.index.json",
   ).expect("hub_load_safetensors failed!");
   
   let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(Error::msg).expect("Tokenizer from file failed!");
   let device = if cuda_is_available() { Device::new_cuda(0).expect("create cuda device failed!") } else { Device::Cpu };
   let dtype =  if device.is_cuda() {DType::BF16} else {DType::F32};
   let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device).expect("var builder failed!") };
   let config_filename = repo.get("config.json").expect("get config filename failed!");
   let config = std::fs::read_to_string(config_filename).expect("Read to string failed!");
   let config: Phi3Config = serde_json::from_str(&config).expect("load Phi3Config failed!");
   let phi3 = Phi3::new(&config, vb).expect("create Phi3 failed!");
   
   TextGeneration::new(
 phi3,
        tokenizer,
        299792458u64,
        Some(0.3f64),
        Some(0.9f64),
        1.9f32,
        16usize,
        &device,
   )
}