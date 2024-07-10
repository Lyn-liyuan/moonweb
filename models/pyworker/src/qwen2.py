from moonipc import IpcChannel;
from transformers import TextStreamer
from transformers import AutoModelForCausalLM, AutoTokenizer
import json
import torch

class IpcStreamer(TextStreamer):
    def __init__(
        self, tokenizer: AutoTokenizer, skip_prompt: bool = False, ipc: IpcChannel = None, **decode_kwargs,
    ):
        super().__init__(tokenizer, skip_prompt, **decode_kwargs)
        self.ipc = ipc

    def on_finalized_text(self, text: str, stream_end: bool = False):
        self.ipc.send(text)
        if stream_end:
           self.ipc.send("<|endoftext|>");

def run(ipc_name,model_id):
   
    ipc = IpcChannel(ipc_name);
    
    device = "cuda"
    model = AutoModelForCausalLM.from_pretrained(
          model_id,
          torch_dtype="auto",
          device_map="auto",
          attn_implementation="flash_attention_2",
    )
    tokenizer = AutoTokenizer.from_pretrained(model_id)
    streamer = IpcStreamer(tokenizer, skip_prompt=True, skip_special_tokens=True,ipc=ipc)
    print(f"{model_id} server start!")
    while True:
        request = json.loads(ipc.recv())
        
        if request['cmd'] == "QUIT":
            break
        messages = [{"role": "system", "content": "你是源胖子训练的中文人工智能助手，你善于简短准确地回答各种问题。"}]
        for msg in request['msg_list'] :
            if msg['role']=='User':
               messages.append({"role":"user","content":msg['content']})
            else:
               messages.append({"role":"assistant","content":msg['content']})
        text = tokenizer.apply_chat_template(
                messages,
                tokenize=False,
                add_generation_prompt=True
        )
        model_inputs = tokenizer([text], return_tensors="pt").to(device)
        
        print("model.generate!!")
        model.generate(
            model_inputs.input_ids,
            
            max_new_tokens=512,
            streamer=streamer,
        )
            
