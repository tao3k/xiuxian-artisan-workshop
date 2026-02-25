import os
import urllib.request
import json
import traceback

def test():
    key = os.environ.get('MINIMAX_API_KEY')
    print("Key exists:", bool(key), "Length:", len(key) if key else 0)
    
    url = 'https://api.minimax.io/v1/chat/completions'
    data = json.dumps({'model': 'MiniMax-Text-01', 'messages': [{'role': 'user', 'content': 'hi'}]}).encode('utf-8')
    req = urllib.request.Request(url, data=data, headers={'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'})
    
    try:
        res = urllib.request.urlopen(req)
        print("Success:", res.read().decode())
    except urllib.error.HTTPError as e:
        print("HTTPError:", e.code, e.read().decode())
    except Exception as e:
        print("Error:", e)
        traceback.print_exc()

test()
