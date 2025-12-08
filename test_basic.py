#!/usr/bin/env python3
"""
Basic MCP server test - verifies initialize works correctly.
The server may exit after initialized notification (this appears to be expected behavior for serve_server).
"""

import json
import subprocess
import sys
import time
import os


def test_initialize():
    """Test that server responds correctly to initialize request."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    server_path = os.path.join(script_dir, "target", "release", "mcp_luna_history")
    if not os.path.exists(server_path):
        server_path = os.path.join(script_dir, "target", "debug", "mcp_luna_history")
    
    if not os.path.exists(server_path):
        print(f"Error: Server binary not found")
        return False
    
    print(f"Testing server: {server_path}\n")
    
    process = subprocess.Popen(
        [server_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    time.sleep(0.3)
    
    try:
        # Send initialize request
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }
        
        request_json = json.dumps(init_request) + "\n"
        process.stdin.write(request_json)
        process.stdin.flush()
        
        time.sleep(0.2)
        response_line = process.stdout.readline()
        
        if not response_line:
            print("✗ No response from server")
            return False
        
        response = json.loads(response_line.strip())
        
        # Verify response
        assert response.get("jsonrpc") == "2.0"
        assert response.get("id") == 1
        assert "result" in response
        assert response["result"].get("protocolVersion") == "2024-11-05"
        assert "capabilities" in response["result"]
        assert "serverInfo" in response["result"]
        
        print("✓ Initialize test PASSED")
        print(f"  Protocol Version: {response['result']['protocolVersion']}")
        print(f"  Server: {response['result']['serverInfo'].get('name')} v{response['result']['serverInfo'].get('version')}")
        print(f"  Instructions: {response['result'].get('instructions', '')[:60]}...")
        
        return True
        
    except Exception as e:
        print(f"✗ Test failed: {e}")
        return False
    finally:
        process.terminate()
        process.wait(timeout=2)


if __name__ == "__main__":
    success = test_initialize()
    sys.exit(0 if success else 1)

