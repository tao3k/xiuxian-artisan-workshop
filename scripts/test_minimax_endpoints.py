import os
import urllib.request
import json

key = os.environ.get("MINIMAX_API_KEY")
print(f"Key present: {bool(key)}")

endpoints = [
    "https://api.minimax.chat/v1/chat/completions",
    "https://api.minimax.io/v1/chat/completions",
]

models = [
    "MiniMax-Text-01",
    "abab6.5s-chat",
    "MiniMax-M2.1",
    "MiniMax-M2.5",
]

for url in endpoints:
    for model in models:
        data = json.dumps({
            "model": model,
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 10
        }).encode("utf-8")
        
        req = urllib.request.Request(
            url, 
            data=data, 
            headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"}
        )
        
        try:
            res = urllib.request.urlopen(req)
            print(f"✅ {url} | {model} -> SUCCESS: {res.read().decode()[:50]}")
        except urllib.error.HTTPError as e:
            err = e.read().decode()
            print(f"❌ {url} | {model} -> {e.code}: {err}")
        except Exception as e:
            print(f"❌ {url} | {model} -> Error: {e}")

