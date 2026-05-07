#!/bin/bash
IMG_B64=$(base64 -w 0 /home/pi/test_image.png)
echo "{
  \"prompt\": \"<|im_start|>user\\n<image>\\nDescribe this image.<|im_end|>\\n<|im_start|>assistant\\n\",
  \"image_data\": [{\"data\": \"$IMG_B64\", \"id\": 10}],
  \"n_predict\": 128
}" > /home/pi/test_payload.json

curl -s http://localhost:8080/completion -H "Content-Type: application/json" -d @/home/pi/test_payload.json
