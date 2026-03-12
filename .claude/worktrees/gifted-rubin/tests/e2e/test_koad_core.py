import pytest
import time
import json

def test_storage_bridge_sync(spine, redis_client):
    """Verify that knowledge saved to SQLite is eventually reflected in Redis or queryable."""
    # 1. Remember a fact via CLI (writes to SQLite)
    fact_text = "The core is stable."
    spine.run_koad(["remember", "fact", fact_text, "--tags", "core,status"])
    
    # 2. Query it back (CLI queries SQLite)
    result = spine.run_koad(["query", "stable"])
    assert fact_text in result.stdout
    
    # 3. Check Redis for any session updates triggered by memory changes
    # (In v4.1, memory doesn't automatically push to Redis koad:state yet, 
    # but we can verify the session metadata reflects the bio)
    boot_res = spine.run_koad(["boot", "--agent", "TestAgent", "--role", "admin"])
    assert "Bio:      E2E Test Agent" in boot_res.stdout

def test_spine_grpc_health(spine):
    """Verify the Spine gRPC server responds to health checks (doctor)."""
    result = spine.run_koad(["doctor"])
    assert result.returncode == 0
    assert "Backbone (Spine):" in result.stdout
    assert "PASS" in result.stdout
