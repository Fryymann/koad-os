import pytest
import asyncio
import websockets
import json
import time
from pathlib import Path

@pytest.mark.asyncio
async def test_full_stack_session_loopback(gateway):
    """
    Verify the complete loop:
    1. CLI: koad boot
    2. Spine: Receives boot, updates Redis, publishes to koad:sessions
    3. Gateway: Normalizes message, broadcasts via WebSocket
    4. Test: Receives message from WebSocket and verifies session ID
    """
    uri = f"ws://127.0.0.1:{gateway.gateway_port}/ws/fabric"
    
    async with websockets.connect(uri) as websocket:
        # 1. Initial Sync might be first
        msg = await websocket.recv()
        data = json.loads(msg)
        assert data["type"] == "SYSTEM_SYNC"
        
        # 2. Trigger koad boot from CLI
        boot_res = gateway.run_koad(["boot", "--agent", "LoopbackAgent"])
        assert boot_res.returncode == 0
        
        # Extract session ID from stdout
        # stdout: "Session:  <id>"
        session_id = None
        for line in boot_res.stdout.splitlines():
            if "Session:" in line:
                session_id = line.split("Session:")[1].strip()
                break
        
        assert session_id is not None
        
        # 3. Listen for SESSION_UPDATE on WebSocket
        # We might get telemetry stats first, so we loop a bit
        found = False
        print(f"Waiting for session {session_id} on WebSocket...")
        for i in range(20):
            try:
                msg = await asyncio.wait_for(websocket.recv(), timeout=2.0)
                try:
                    data = json.loads(msg)
                except json.JSONDecodeError:
                    print(f"Skipping non-JSON message: {msg}")
                    continue
                    
                print(f"WS Msg {i}: {data.get('type')}")
                if data.get("type") == "SESSION_UPDATE":
                    payload = data.get("payload", {})
                    print(f"  Got Session: {payload.get('session_id')}")
                    if payload.get("session_id") == session_id:
                        assert payload["identity"]["name"] == "LoopbackAgent"
                        # CRITICAL GAP CHECK: Ensure normalized fields are present
                        assert "environment" in payload
                        assert "last_heartbeat" in payload
                        found = True
                        break
            except asyncio.TimeoutError:
                break
        
        assert found, f"Session {session_id} never appeared in WebSocket stream"

@pytest.mark.asyncio
async def test_gateway_binary_staleness_loopback(gateway):
    """
    Verify that if we simulate a binary update, koad doctor catches it.
    This ensures our 'Doctor' isn't blind to process drift.
    """
    # 1. Run doctor - baseline
    res = gateway.run_koad(["doctor"])
    # Ignore existing host processes that might be stale
    
    # 2. Touch the gateway binary to simulate an update
    gateway_bin = gateway.bin_dir / "kgateway"
    # Ensure it exists
    assert gateway_bin.exists()
    
    # Set mtime to future
    os.utime(gateway_bin, (time.time() + 100, time.time() + 100))
    
    # 3. Run doctor again
    res = gateway.run_koad(["doctor"])
    # It must specifically report our test kgateway as stale
    assert "STALE kgateway" in res.stdout
    assert "Binary updated since process started" in res.stdout

import os
