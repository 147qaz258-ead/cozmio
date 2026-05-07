import base64
import json

img_path = r'C:\Users\29913\.gemini\antigravity\brain\3d51fbf5-c52a-4946-b5d9-43e76e4ecac3\vision_test_image_1777997580359.png'
with open(img_path, 'rb') as f:
    b64 = base64.b64encode(f.read()).decode()

payload = {
    "prompt": "<|im_start|>user\n<image>\nDescribe this image in detail.<|im_end|>\n<|im_start|>assistant\n",
    "image_data": [{"data": b64, "id": 0}],
    "n_predict": 128
}

with open(r'd:\C_Projects\Agent\cozmio\cozmio\cozmio-box-worker\scripts\test_vision.json', 'w') as f:
    json.dump(payload, f)
