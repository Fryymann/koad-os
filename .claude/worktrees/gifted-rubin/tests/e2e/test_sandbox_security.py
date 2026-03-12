import pytest
import json
import time

def test_admin_sandbox_execution(spine, redis_client):
    """Verify that an Admin can execute commands via the Spine."""
    # Run a simple echo command
    intent = {
        "type": "execute",
        "data": {
            "identity": "admin",
            "command": "echo 'Hello KoadOS'",
            "args": [],
            "working_dir": None,
            "env_vars": {}
        }
    }
    redis_client.publish("koad:commands", json.dumps(intent))
    
    # Verify execution in the event stream
    found_success = False
    for _ in range(50):
        events = redis_client.xrange("koad:events:stream", "-", "+")
        for _, fields in events:
            if fields.get("message") == "TASK_LIFECYCLE":
                meta = json.loads(fields.get("metadata", "{}"))
                if meta.get("status") == "SUCCESS" and "echo" in meta.get("command", ""):
                    found_success = True
                    break
        if found_success: break
        time.sleep(0.1)
        
    assert found_success, "Kernel should have executed the command for Admin"

def test_developer_sandbox_denial(spine, redis_client):
    """Verify that a Developer role is blocked from blacklisted commands."""
    # 1. Elevate to Developer role in koad.json
    koad_json = spine.koad_home / "koad.json"
    config = json.loads(koad_json.read_text())
    config["identity"]["role"] = "Developer"
    koad_json.write_text(json.dumps(config))
    
    # 2. Attempt a blacklisted command (sudo) via the Spine's command bus
    intent = {
        "type": "execute",
        "data": {
            "identity": "developer",
            "command": "sudo rm -rf /",
            "args": [],
            "working_dir": None,
            "env_vars": {}
        }
    }
    redis_client.publish("koad:commands", json.dumps(intent))
    
    # 3. Verify rejection in the event stream
    found_rejection = False
    for _ in range(50):
        events = redis_client.xrange("koad:events:stream", "-", "+")
        for _, fields in events:
            if fields.get("message") == "TASK_REJECTED":
                meta = json.loads(fields.get("metadata", "{}"))
                if "Policy Violation" in meta.get("error", ""):
                    found_rejection = True
                    break
        if found_rejection: break
        time.sleep(0.1)
        
    assert found_rejection, "Kernel should have rejected the sudo command for Developer role"
