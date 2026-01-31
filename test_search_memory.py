#!/usr/bin/env python3
"""Quick test: run MCP server and call search_memory with keywords ['moltbook','security','info']."""

import json
import os
import subprocess
import sys
import time

DB_PATH = "/home/digit1024/.local/share/cosmic_llm/conversations.db"
QUERY = "moltbook security info"

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    server_path = os.path.join(script_dir, "target", "release", "mcp_luna_history")
    if not os.path.exists(server_path):
        print(f"Error: Build first: cargo build --release")
        sys.exit(1)
    if not os.path.exists(DB_PATH):
        print(f"Error: DB not found: {DB_PATH}")
        sys.exit(1)

    env = os.environ.copy()
    env["COSMIC_LLM_DB_PATH"] = DB_PATH

    proc = subprocess.Popen(
        [server_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0,
        env=env,
    )
    request_id = 1

    def send(method, params=None):
        nonlocal request_id
        msg = {"jsonrpc": "2.0", "id": request_id, "method": method}
        if params:
            msg["params"] = params
        request_id += 1
        proc.stdin.write(json.dumps(msg) + "\n")
        proc.stdin.flush()
        time.sleep(0.15)
        line = proc.stdout.readline()
        return json.loads(line.strip()) if line else None

    def notify(method, params=None):
        msg = {"jsonrpc": "2.0", "method": method}
        if params:
            msg["params"] = params
        proc.stdin.write(json.dumps(msg) + "\n")
        proc.stdin.flush()
        time.sleep(0.2)

    print(f"DB: {DB_PATH}")
    print(f"Query: '{QUERY}'")
    print("-" * 50)

    r = send("initialize", {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test", "version": "1.0"},
    })
    if "error" in r:
        print("Initialize error:", r["error"])
        proc.terminate()
        sys.exit(1)
    print("✓ initialize")

    notify("notifications/initialized")
    print("✓ initialized")

    r = send("tools/call", {
        "name": "search_memory",
        "arguments": {"keywords": QUERY.split()},
    })
    proc.terminate()

    if "error" in r:
        print("Error:", r["error"])
        sys.exit(1)

    content = r.get("result", {}).get("content", [])
    if not content:
        print("No content in result:", json.dumps(r, indent=2))
        sys.exit(0)

    text = content[0].get("text", "{}")
    data = json.loads(text)
    items = data.get("items", [])
    print(f"\nsearch_memory result: {len(items)} item(s)")
    for i, m in enumerate(items):
        print(f"\n[{i+1}] id={m.get('id')} importance={m.get('importance')}")
        print(f"    {m.get('content', '')[:200]}")
        if m.get("category"):
            print(f"    category: {m['category']}")

    if not items:
        print("(no matches)")

if __name__ == "__main__":
    main()
