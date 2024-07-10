from llava.model.builder import load_pretrained_model
from llava.mm_utils import get_model_name_from_path, process_images, tokenizer_image_token
from llava.constants import IMAGE_TOKEN_INDEX, DEFAULT_IMAGE_TOKEN, DEFAULT_IM_START_TOKEN, DEFAULT_IM_END_TOKEN, IGNORE_INDEX
from llava.conversation import conv_templates, SeparatorStyle

from PIL import Image
import base64
import io
import requests
import copy
import torch
import json
from transformers import TextStreamer,AutoTokenizer
from moonipc import IpcChannel;


class IpcStreamer(TextStreamer):
    def __init__(
        self, tokenizer: AutoTokenizer, skip_prompt: bool = False, ipc: IpcChannel = None, **decode_kwargs,
    ):
        super().__init__(tokenizer, skip_prompt, **decode_kwargs)
        self.ipc = ipc

    def on_finalized_text(self, text: str, stream_end: bool = False):
        self.ipc.send(text)
        if stream_end:
           self.ipc.send("<|endoftext|>")

def run(ipc_name,model_id = "lmms-lab/llama3-llava-next-8b"):

    ipc = IpcChannel(ipc_name);
    model_name = "llava_llama3"
    device = "cuda"
    device_map = "auto"
    tokenizer, model, image_processor, max_length = load_pretrained_model(model_id, None, model_name, device_map=device_map,repeat_penalty=1.8,repeat_last_n=64) 
    streamer = IpcStreamer(tokenizer, skip_prompt=True, skip_special_tokens=True,ipc=ipc)
    conv_template = "llava_llama_3"

    conv = copy.deepcopy(conv_templates[conv_template])
    
    print(f"{model_id} server start!")
    while True:
        request = json.loads(ipc.recv())
        
        if request['cmd'] == "QUIT":
            break
        conv.messages.clear()
        image_list = []
        has_image = False
        for msg in request['msg_list']:
            if msg['img'] is not None:
               base64_string = msg['img']
               if base64_string.startswith('data:image'):
                   base64_string = base64_string.split(',')[1]     
               image_data = base64.b64decode(base64_string)
               image = Image.open(io.BytesIO(image_data))
               image_list.append(image)
               has_image = True
            else:
                image_tag = ""
                if has_image:
                   image_tag = DEFAULT_IMAGE_TOKEN+"\n"
                has_image = False
                if msg['role']=='User':
                    conv.append_message(conv.roles[0], image_tag+msg['content'])
                else:
                    conv.append_message(conv.roles[1], msg['content'])
         
        conv.append_message(conv.roles[1], None)
        prompt_question = conv.get_prompt()
        print(prompt_question)
        input_ids = tokenizer_image_token(prompt_question, tokenizer, IMAGE_TOKEN_INDEX, return_tensors="pt").unsqueeze(0).to(device)
        
        image_tensor = process_images(image_list, image_processor, model.config)
        image_tensor = [_image.to(dtype=torch.float16, device=device) for _image in image_tensor]
        image_sizes = [image.size for image in image_list]

        cont = model.generate(
            input_ids,
            images=image_tensor,
            image_sizes=image_sizes,
            do_sample=True,
            top_p=0.95,
            temperature=0.5,
            pad_token_id=tokenizer.eos_token_id,
            max_new_tokens=256,
            streamer = streamer,
        )
