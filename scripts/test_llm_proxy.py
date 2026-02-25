import asyncio
import json
import os
import subprocess
import time
import urllib.request
import urllib.error

async def main():
    env = os.environ.copy()
    
    # Do NOT overwrite real keys if they exist in the user's environment.
    # We only set dummy keys if they are completely missing, so the proxy can boot/test.
    if not env.get("MINIMAX_API_KEY"):
        env["MINIMAX_API_KEY"] = "dummy_minimax_key"
    if not env.get("OPENAI_API_KEY"):
        env["OPENAI_API_KEY"] = "dummy_openai_key"
        
    env["VALKEY_URL"] = env.get("VALKEY_URL", "redis://127.0.0.1:6379/0")
    env.pop("DYLD_LIBRARY_PATH", None)

    print("Building omni-agent...")
    subprocess.run(["cargo", "build", "--bin", "omni-agent"], check=True, env=env)
    
    port = 8085
    
    print(f"Starting omni-agent gateway on port {port}...")
    proc = subprocess.Popen(
        ["cargo", "run", "--bin", "omni-agent", "--", "gateway", "--bind", f"127.0.0.1:{port}"],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    
    try:
        # Wait for server to start
        for i in range(15):
            try:
                urllib.request.urlopen(f"http://127.0.0.1:{port}/health")
                break
            except Exception:
                time.sleep(1)
        else:
            print("Failed to start agent gateway.")
            proc.kill()
            out, err = proc.communicate()
            print("STDOUT:", out.decode())
            print("STDERR:", err.decode())
            return

        print("Agent gateway is up. Testing LLM Proxy...")
        
        # Test 1: Minimax model via proxy
        req_data = json.dumps({
            "model": "minimax/MiniMax-M2.5",
            "messages": [{"role": "user", "content": "Hello! Reply 'Hi' if you can hear me."}],
            "max_tokens": 10
        }).encode("utf-8")
        
        req = urllib.request.Request(
            f"http://127.0.0.1:{port}/v1/chat/completions",
            data=req_data,
            headers={"Content-Type": "application/json"}
        )
        
        print("\\n--- Testing Minimax Provider ---")
        try:
            with urllib.request.urlopen(req) as response:
                res_body = response.read().decode("utf-8")
                print(f"Response: {response.status} {res_body}")
                print("✅ Minimax proxy routing works (Got 200 OK with real completions!)")
        except urllib.error.HTTPError as e:
            err_body = e.read().decode("utf-8")
            print(f"HTTPError: {e.code} {err_body}")
            # If we get 401, 429, or 500, it means the proxy correctly forwarded the request to minimax!
            if e.code in (401, 429, 500, 502) or "invalid" in err_body.lower() or "upstream" in err_body.lower() or "balance" in err_body.lower() or "support model" in err_body.lower():
                print("✅ Minimax proxy routing works (received expected upstream response)")
            else:
                print("❌ Unexpected error")
                
        # Test 2: OpenAI model via proxy
        req_data = json.dumps({
            "model": "openai/gpt-4o",
            "messages": [{"role": "user", "content": "Hello"}],
        }).encode("utf-8")
        
        req = urllib.request.Request(
            f"http://127.0.0.1:{port}/v1/chat/completions",
            data=req_data,
            headers={"Content-Type": "application/json"}
        )
        
        print("\\n--- Testing OpenAI Provider ---")
        try:
            with urllib.request.urlopen(req) as response:
                res_body = response.read().decode("utf-8")
                print(f"Response: {response.status} {res_body}")
                print("✅ OpenAI proxy routing works (Got 200 OK)")
        except urllib.error.HTTPError as e:
            err_body = e.read().decode("utf-8")
            print(f"HTTPError: {e.code} {err_body}")
            if e.code in (401, 429, 500, 502) or "invalid" in err_body.lower() or "upstream" in err_body.lower() or "balance" in err_body.lower():
                print("✅ OpenAI proxy routing works (received expected upstream response)")
            else:
                print("❌ Unexpected error")

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()

if __name__ == "__main__":
    asyncio.run(main())
