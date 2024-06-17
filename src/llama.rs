use core::str;

use anyhow::{Error, Result};
use crate::data::{Role,Message};
use candle_core::utils::cuda_is_available;
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use hf_hub::{api::sync::Api, Repo, RepoType};

use candle_transformers::models::llama as model;
use model::{Llama, LlamaConfig, Config};

use crate::token_output_stream::TokenOutputStream;
use crate::model::TextGenModel;
use tokenizers::Tokenizer;


const EOS_TOKEN: &str = "<|eot_id|>";

pub struct TextGeneration {
    model: Llama,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    eos_token_id: Option<u32>,
    config: Config,
    repeat_penalty: f32,
    repeat_last_n: usize,
}
impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: Llama,
        tokenizer: Tokenizer,
        eos_token_id: Option<u32>,
        config: Config,
        seed: u64,
        temp: f64,
        top_p: f64,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
    ) -> Self {
        let logits_processor = LogitsProcessor::from_sampling(
            seed,
            Sampling::TopP {
                p: top_p,
                temperature: temp,
            },
        );
        Self {
            model,
            tokenizer: TokenOutputStream::new(tokenizer),
            eos_token_id: eos_token_id,
            config: config,
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
        }
    }
}

impl TextGenModel for TextGeneration {
    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<String, Error> {
        self.tokenizer.clear();
        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(Error::msg)?
            .get_ids()
            .to_vec();
        let mut cache = model::Cache::new(true, DType::F32, &self.config, &self.device)?;
        println!("starting the inference loop");
        print!("{prompt}");
        let mut start_gen = std::time::Instant::now();
        let mut index_pos = 0;
        let mut token_generated = 0;

        for index in 0..sample_len {
            let (context_size, context_index) = if cache.use_kv_cache && index > 0 {
                (1, index_pos)
            } else {
                (tokens.len(), 0)
            };
            if index == 1 {
                start_gen = std::time::Instant::now()
            }
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, context_index, &mut cache)?;
            let logits = logits.squeeze(0)?;
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
            index_pos += ctxt.len();
    
            let next_token = self.logits_processor.sample(&logits)?;
            token_generated += 1;
            tokens.push(next_token);
    
            if Some(next_token) == self.eos_token_id {
                break;
            }
            self.tokenizer.put_token(next_token);
            // if let Some(t) = self.tokenizer.next_token(next_token)? {
            //     print!("{t}");
            //     std::io::stdout().flush()?;
            // }

        }
        let content = self.tokenizer.decode_all().expect("decode text failed!");
        if let Some(rest) = self.tokenizer.decode_rest().map_err(Error::msg)? {
            print!("{rest}");
        }
        let dt = start_gen.elapsed();
        println!(
            "\n\n{} tokens generated ({} token/s)\n",
            token_generated,
            (token_generated - 1) as f64 / dt.as_secs_f64(),
        );
        Ok(content)
    }
    fn messages_chat_template(&self,msg_list: &Vec<Message>,system_prompt:&str)->String {
        let mut history = String::new();
        history.push_str("<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n");
        history.push_str(format!("{}<|eot_id|>",system_prompt).as_str());
        
        for msg in msg_list {
            if msg.role == Role::User {
                history.push_str("<|start_header_id|>user<|end_header_id|>\n\n");
            } else {
                history.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
            }
            history.push_str(msg.content.as_str());
            history.push_str("<|eot_id|>\n");
            
        }
        history.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
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

// pub fn load()->TextGeneration {
//     load_model("meta-llama/Meta-Llama-3-8B-Instruct",0.6f64,0.9f64)
// }



pub fn load_model(model_id:&str, temp: f64,
    top_p: f64,) -> impl TextGenModel {
    
    let revision = String::from("main");
    
    let device = if cuda_is_available() {
        Device::new_cuda(0).expect("create cuda device failed!")
    } else {
        Device::Cpu
    };
    
    let dtype = DType::F32;
    let api = Api::new().expect("create Api failed!");
    let api = api.repo(Repo::with_revision(model_id.to_string(), RepoType::Model, revision));

    let tokenizer_filename = api
        .get("tokenizer.json")
        .expect("get tokenizer.json failed!");
    let config_filename = api.get("config.json").expect("get config.json failed!");
    let config: LlamaConfig =
        serde_json::from_slice(&std::fs::read(config_filename).expect("read config file failed!"))
            .expect("serde_json from slice config file failed!");
    let config = config.into_config(false);
    let filenames = hub_load_safetensors(&api, "model.safetensors.index.json")
        .expect("hub_load_safetensors failed!");

    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)
            .expect("var builder failed!")
    };
    let llama = Llama::load(vb, &config).expect("llama load failed!");
    let tokenizer = Tokenizer::from_file(tokenizer_filename)
        .map_err(Error::msg)
        .expect("load tokenzier failed!");

    let eos_token_id = tokenizer.token_to_id(EOS_TOKEN);

    
    TextGeneration::new(
        llama,
        tokenizer,
        eos_token_id,
        config,
        299792458u64,
        temp,
        top_p,
        1.8f32,
        16usize,
        &device,
    )
}
