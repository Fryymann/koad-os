import pytest
import json
import time

def test_koad_doctor_report(spine):
    """Verify that koad doctor reports on system health."""
    result = spine.run_koad(["doctor"])
    assert result.returncode == 0
    assert "Neural Link & Grid Integrity" in result.stdout
    assert "PASS" in result.stdout
    assert "Backbone (Spine):" in result.stdout
    assert "PASS" in result.stdout
    assert "Memory Bank (SQLite):" in result.stdout

def test_governance_clean_execution(spine, redis_client):
    """Verify that a Governance::Clean intent is routed and executed."""
    # repo-clean.py requires a git repo to run git commands
    import subprocess
    subprocess.run(["git", "init"], cwd=spine.koad_home, check=True)

    # Give kspine a moment to subscribe
    time.sleep(2)

    # 1. Dispatch Governance Intent
    intent = {
        "type": "governance",
        "data": {
            "action": "clean",
            "target": None
        }
    }
    redis_client.publish("koad:commands", json.dumps(intent))

    # 2. Wait for KCM execution event
    found_success = False
    for _ in range(50):
        events = redis_client.xrange("koad:events:stream", "-", "+")
        for _, fields in events:
            if fields.get("source") == "engine:kcm" and fields.get("message") == "GOVERNANCE_EXECUTION":
                meta = json.loads(fields.get("metadata", "{}"))
                if meta.get("action") == "clean" and meta.get("status") == "SUCCESS":
                    found_success = True
                    break
        if found_success: break
        time.sleep(0.1)

    assert found_success, "KCM should have executed and logged the CLEAN governance action"
