use anyhow::{Error as E, Result};
use candle_core::utils::cuda_is_available;
use clap::Parser;

use candle_transformers::models::qwen2::{Config as ConfigBase, ModelForCausalLM as ModelBase};

use candle_core::{DType, Device, Tensor};
use moonweb::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use moonweb::ipc::{accept,OutputStream};
use moonweb::data::{Request,Message,Role};

struct TextGeneration {
    model: ModelBase,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: ModelBase,
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

    fn run(&mut self,output: &impl OutputStream,prompt: &str, sample_len: usize) -> Result<()> {
        self.model.clear_kv_cache();
        self.tokenizer.clear();
        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                print!("{t}")
            }
        }
        

        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("<|endoftext|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|endoftext|> token"),
        };
        let start_gen = std::time::Instant::now();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
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

            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;
            if next_token == eos_token {
                break;
            }
            if let Some(t) = self.tokenizer.next_token(next_token)? {
                //print!("{t}");
                //std::io::stdout().flush()?;
                output.write(format!("{t}")).unwrap();
            }
        }
        output.end().unwrap();
        let dt = start_gen.elapsed();
        if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
            print!("{rest}");
        }
        
        println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            generated_tokens as f64 / dt.as_secs_f64(),
        );
        Ok(())
    }
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    
    #[clap(short, long)]
    server: String,

    #[clap(short, long)]
    ipc_name: String,
    
    #[arg(long)]
    temperature: Option<f64>,

    /// Nucleus sampling probability cutoff.
    #[arg(long)]
    top_p: Option<f64>,

    /// The seed to use when generating random samples.
    #[arg(long, default_value_t = 299792458)]
    seed: u64,

    /// The length of the sample to generate (in tokens).
    #[arg(long, short = 'n', default_value_t = 10000)]
    sample_len: usize,

    #[arg(long,default_value = "Qwen/Qwen2-7B")]
    model_id: Option<String>,

    #[arg(long, default_value = "main")]
    revision: String,

    #[arg(long)]
    tokenizer_file: Option<String>,

    #[arg(long)]
    weight_files: Option<String>,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    #[arg(long, default_value_t = 1.1)]
    repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    #[arg(long, default_value_t = 64)]
    repeat_last_n: usize,
}



fn messages_chat_template(msg_list: &Vec<Message>,system_prompt:&str)->String {
    let mut history = String::new();
    history.push_str("<|im_start|>system\n");
    history.push_str(system_prompt);
    history.push_str("<|im_end|>\n");
    for msg in msg_list {
        history.push_str("<|im_start|>");
        if msg.role == Role::User {
           history.push_str("user\n");
        } else {
           history.push_str("assistant\n");
        }
        history.push_str(msg.content.as_str());
        history.push_str("<|im_end|>\n");
    }
    history.push_str("<|im_start|>assistant\n");
    history
}

fn main() -> Result<()> {
    
    let args = Args::parse();
    let start = std::time::Instant::now();
    let api = Api::new()?;
    let model_id = "Qwen/Qwen2-1.5B-Instruct".to_string();
    let repo = api.repo(Repo::with_revision(
        model_id.clone(),
        RepoType::Model,
        "main".to_string(),
    ));
    let tokenizer_filename = repo.get("tokenizer.json")?;
    let filenames = vec![repo.get("model.safetensors")?];
    println!("retrieved the files in {:?}", start.elapsed());
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

    let start = std::time::Instant::now();
    let config_file = repo.get("config.json")?;
    let device = if cuda_is_available() {
        Device::new_cuda(0).expect("create cuda device failed!")
    } else {
        Device::Cpu
    };
    
    let dtype = if device.is_cuda() {
        DType::BF16
    } else {
        DType::F32
    };
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
    let model =  {
            let config: ConfigBase = serde_json::from_str(&std::fs::read_to_string(config_file)?)?;
            ModelBase::new(&config, vb)?
    };

    println!("loaded the model in {:?}", start.elapsed());
    let temp = args.temperature.unwrap_or_else(|| 0.3f64);
    let top_p = args.top_p.unwrap_or_else(|| 0.95f64);
    let mut pipeline = TextGeneration::new(
        model,
        tokenizer,
        299792458u64,
        Some(temp),
        Some(top_p),
        1.8f32,
        64usize,
        &device,
    );
    let ipc_name = args.ipc_name;
    let (receiver,sender) = accept(ipc_name);
    println!("{} server start!",model_id);
    loop {
        let msg = receiver.recv().unwrap();
        if let Ok(req) = serde_json::from_str::<Request>(msg.as_str()) {
            if req.cmd.eq("QUIT") {
                    break;
            }
            let prompt = messages_chat_template(&req.msg_list,"你是源胖子开发的AI助手，你善于回答科普问题。");
            
            pipeline.run(&sender,prompt.as_str(), 1000usize)?;
        }
        
    }
    
    Ok(())
}