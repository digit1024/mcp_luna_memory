#!/usr/bin/env python3
"""
Simple test script for MCP server - tests initialize and tools/list in one session.
"""

import json
import subprocess
import sys
import time


def test_mcp_server():
    """Test MCP server with full protocol flow."""
    import os
    
    # Determine server path
    script_dir = os.path.dirname(os.path.abspath(__file__))
    server_path = os.path.join(script_dir, "target", "debug", "mcp_luna_history")
    release_path = os.path.join(script_dir, "target", "release", "mcp_luna_history")
    if os.path.exists(release_path):
        server_path = release_path
    
    if not os.path.exists(server_path):
        print(f"Error: Server binary not found at {server_path}")
        print("Please build the server first: cargo build")
        return False
    
    print(f"Starting server: {server_path}\n")
    
    # Start server
    process = subprocess.Popen(
        [server_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )
    
    time.sleep(0.3)  # Give server time to start
    
    try:
        # Test 1: Initialize
        print("="*60)
        print("TEST 1: Initialize")
        print("="*60)
        
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        }
        
        request_json = json.dumps(init_request) + "\n"
        print(f"→ Sending: {json.dumps(init_request, indent=2)}\n")
        process.stdin.write(request_json)
        process.stdin.flush()
        
        time.sleep(0.2)
        response_line = process.stdout.readline()
        if not response_line:
            print("✗ No response from server")
            return False
        
        response = json.loads(response_line.strip())
        print(f"← Response: {json.dumps(response, indent=2)}\n")
        
        if "error" in response:
            print(f"✗ Server error: {response['error']}")
            return False
        
        assert response.get("jsonrpc") == "2.0"
        assert "result" in response
        assert response["result"].get("protocolVersion") == "2024-11-05"
        print("✓ Initialize test passed\n")
        
        # Test 2: Initialized notification
        print("="*60)
        print("TEST 2: Initialized Notification")
        print("="*60)
        
        init_notification = {
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }
        
        notification_json = json.dumps(init_notification) + "\n"
        print(f"→ Sending: {json.dumps(init_notification, indent=2)}\n")
        process.stdin.write(notification_json)
        process.stdin.flush()
        
        time.sleep(0.3)
        
        # Test 3: List tools
        print("="*60)
        print("TEST 3: List Tools")
        print("="*60)
        
        # Check if server is still running
        if process.poll() is not None:
            print(f"✗ Server exited with code {process.returncode}")
            stderr_output = process.stderr.read()
            if stderr_output:
                print(f"\nServer stderr:\n{stderr_output}")
            return False
        
        tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }
        
        request_json = json.dumps(tools_request) + "\n"
        print(f"→ Sending: {json.dumps(tools_request, indent=2)}\n")
        process.stdin.write(request_json)
        process.stdin.flush()
        
        time.sleep(0.3)
        response_line = process.stdout.readline()
        if not response_line:
            print("✗ No response from server")
            return False
        
        response = json.loads(response_line.strip())
        print(f"← Response: {json.dumps(response, indent=2)}\n")
        
        if "error" in response:
            print(f"⚠ Server returned error: {response['error']}")
            return False
        
        assert response.get("jsonrpc") == "2.0"
        assert "result" in response
        assert "tools" in response["result"]
        
        tools = response["result"]["tools"]
        print(f"✓ Found {len(tools)} tools:")
        for tool in tools:
            print(f"  - {tool.get('name')}: {tool.get('description', '')[:50]}...")
        
        expected_tools = [
            "search_conversations",
            "get_conversation", 
            "search_conversation_titles",
            "list_conversations",
            "get_message"
        ]
        
        tool_names = [t["name"] for t in tools]
        missing = [t for t in expected_tools if t not in tool_names]
        if missing:
            print(f"\n⚠ Missing tools: {missing}")
        else:
            print("\n✓ All expected tools found!")
        
        print("\n" + "="*60)
        print("✓ ALL TESTS PASSED!")
        print("="*60)
        return True
        
    except Exception as e:
        print(f"\n✗ ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        # Clean up
        try:
            process.terminate()
            process.wait(timeout=2)
        except:
            process.kill()
        
        # Print any remaining stderr
        stderr_output = process.stderr.read()
        if stderr_output:
            print("\n--- Server stderr output ---")
            print(stderr_output)


if __name__ == "__main__":
    success = test_mcp_server()
    sys.exit(0 if success else 1)

