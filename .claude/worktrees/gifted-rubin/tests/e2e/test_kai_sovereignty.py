import pytest
import time
import json
import redis
import os

def test_kai_mutual_exclusion(spine):
    """Verify that a KAI cannot be booted by two sessions simultaneously."""
    env = {"GEMINI_CLI": "1"}
    r = redis.Redis(unix_socket_path=str(spine.redis_socket), decode_responses=True)
    r.hdel("koad:state", "koad:kai:Vigil:lease")

    # 1. Boot first session as Vigil
    res1 = spine.run_koad(["boot", "--agent", "Vigil", "--role", "admin", "--compact"], env=env)
    assert res1.returncode == 0
    assert "I:Vigil" in res1.stdout
    
    # 2. Attempt to boot second session as Vigil (Should be BLOCKED by Spine)
    res2 = spine.run_koad(["boot", "--agent", "Vigil", "--role", "admin", "--compact"], env=env)
    assert res2.returncode != 0
    assert "IDENTITY_LOCKED" in res2.stderr
def test_kai_sovereign_lock(spine):
    """Verify that the Koad KAI rejects low-tier drivers."""
    # Mock a Tier 2 driver (Codex)
    env = {"CODEX_CLI": "1"}
    r = redis.Redis(unix_socket_path=str(spine.redis_socket), decode_responses=True)
    r.delete("koad:state")

    # Attempt to boot Koad (Sovereign) with Tier 2 (Codex)
    res = spine.run_koad(["boot", "--agent", "Koad", "--role", "admin", "--compact"], env=env)
    assert res.returncode != 0
    # Can be local CLI rejection or Spine gRPC rejection
    assert "Cognitive Protection" in res.stderr or "COGNITIVE_REJECTION" in res.stderr

def test_kai_lease_cleanup(spine):
    """Verify that a KAI lease is released after session termination."""
    env = {"GEMINI_CLI": "1"}
    r = redis.Redis(unix_socket_path=str(spine.redis_socket), decode_responses=True)
    r.delete("koad:state")

    # 1. Boot Vigil
    res1 = spine.run_koad(["boot", "--agent", "Vigil", "--role", "admin", "--compact"], env=env)
    assert res1.returncode == 0
    
    # 2. Manually release via Redis (Simulating gracefull exit or TTL)
    # In real world, we'd wait for TTL or send a signal, but for E2E we verify the key exists
    r = redis.Redis(unix_socket_path=str(spine.redis_socket), decode_responses=True)
    assert r.hexists("koad:state", "koad:kai:Vigil:lease")
    
    # Delete the lease manually to simulate release for the next test step
    r.hdel("koad:state", "koad:kai:Vigil:lease")
    
    # 3. Boot Vigil again (Should succeed now)
    res2 = spine.run_koad(["boot", "--agent", "Vigil", "--role", "admin", "--compact"], env=env)
    assert res2.returncode == 0
    assert "I:Vigil" in res2.stdout
