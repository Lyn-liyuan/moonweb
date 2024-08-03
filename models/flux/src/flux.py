import torch
from diffusers import FluxPipeline
from PIL import Image
import base64
import io
import requests
import copy
import torch
import json
import time
import random
import string

from moonipc import IpcChannel;

def generate_image_filename():
    timestamp = str(int(time.time()))
    random_chars = ''.join(random.choices(string.ascii_uppercase + string.digits, k=6))
    return f"{timestamp}{random_chars}.png"

pipe = FluxPipeline.from_pretrained("black-forest-labs/FLUX.1-schnell", torch_dtype=torch.bfloat16)
pipe.enable_model_cpu_offload()



def run(ipc_name,model_id = "black-forest-labs/FLUX.1-schnell"):
    ipc = IpcChannel(ipc_name);
    print(f"{model_id} server start!")
    while True:
        request = json.loads(ipc.recv())
        
        if request['cmd'] == "QUIT":
            break
        msg = request['msg_list'][-1]
        prompt = msg['content']
        image = pipe(
            prompt,
            guidance_scale=0.0,
            output_type="pil",
            num_inference_steps=4,
            max_sequence_length=256,
            generator=torch.Generator("cpu").manual_seed(0)
        ).images[0]
        filename = generate_image_filename()
        image.save(f"dist/images/{filename}")
        ipc.send(f"![{prompt}](/images/{filename})")
        ipc.send("<|endoftext|>")
        
