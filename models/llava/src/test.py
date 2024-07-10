from PIL import Image
import base64
import io

with open("img.txt") as f :
     base64_string = f.read()
     if base64_string.startswith('data:image'):
         base64_string = base64_string.split(',')[1]     
     image_data = base64.b64decode(base64_string)
     image = Image.open(io.BytesIO(image_data))