import pytest
import json
import sqlite3

def test_koad_boot(koad_env):
    """Verify that koad boot runs and generates a valid context."""
    result = koad_env.run_koad(["boot", "--agent", "TestAgent", "--role", "admin"])
    assert result.returncode == 0
    assert "Identity: TestAgent (admin)" in result.stdout
    assert "Bio:      E2E Test Agent" in result.stdout
    assert "Session:" in result.stdout

def test_koad_whoami(koad_env):
    """Verify the whoami command reports the correct identity from koad.json."""
    result = koad_env.run_koad(["whoami"])
    assert result.returncode == 0
    # In whoami it still reads from koad.json legacy config
    assert "TestKoad" in result.stdout
    assert "Admin" in result.stdout

def test_agent_hydration_on_boot(spine):
    """Verify that koad boot initializes a session and receives context."""
    result = spine.run_koad(["boot", "--agent", "TestAgent", "--role", "admin"])
    assert result.returncode == 0
    assert "Identity: TestAgent (admin)" in result.stdout
    # Context hydration check
    assert "[CONTEXT:" in result.stdout

def test_koad_remember_and_query(koad_env, db_conn):
    """Verify that koad remember persists to SQLite and query retrieves it."""
    # 1. Remember a fact
    rem_res = koad_env.run_koad(["remember", "fact", "The spaceship is fueled.", "--tags", "test,fuel"])
    assert rem_res.returncode == 0
    
    # 2. Query the fact
    query_res = koad_env.run_koad(["query", "fueled"])
    assert query_res.returncode == 0
    assert "The spaceship is fueled." in query_res.stdout
    
    # 3. Direct DB check
    cursor = db_conn.cursor()
    cursor.execute("SELECT content FROM knowledge WHERE tags LIKE '%fuel%'")
    row = cursor.fetchone()
    assert row is not None
    assert row[0] == "The spaceship is fueled."

def test_koad_stat(spine, redis_client):
    """Verify that koad stat retrieves metrics from Redis."""
    # 1. Inject mock stats into Redis
    stats = {
        "cpu_usage": 42.5,
        "memory_usage": 1024,
        "skill_count": 5,
        "active_tasks": 2
    }
    redis_client.hset("koad:state", "system_stats", json.dumps(stats))
    
    # 2. Run koad stat
    result = spine.run_koad(["stat"])
    assert result.returncode == 0
    assert "42.5%" in result.stdout
    assert "1024 MB" in result.stdout
    assert "Skills:    5" in result.stdout

def test_koad_auth_output(koad_env):
    """Verify that koad auth correctly labels the active PAT."""
    # Admin role
    result = koad_env.run_koad(["auth"])
    assert result.returncode == 0
    assert "GH:GITHUB_ADMIN_PAT" in result.stdout

def test_koad_board_status_graceful_failure(koad_env):
    """Verify that kboard status handles missing/invalid tokens gracefully."""
    # Provide a dummy token that will fail at the API level
    env = {"GITHUB_PERSONAL_PAT": "dummy_token"}
    result = koad_env.run_koad(["board", "status"], env=env)
    # It might return 1 due to API failure, but shouldn't panic
    assert "Accessing Neural Log" in result.stdout

def test_koad_stat_json(spine, redis_client):
    """Verify that koad stat --json returns raw JSON."""
    stats = {"cpu_usage": 10.0, "memory_usage": 500, "skill_count": 1, "active_tasks": 0}
    redis_client.hset("koad:state", "system_stats", json.dumps(stats))
    
    result = spine.run_koad(["stat", "--json"])
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["cpu_usage"] == 10.0

def test_crew_manifest(spine, redis_client):
    """Verify that koad crew shows live personnel and filters stale ones."""
    from datetime import datetime, timedelta, timezone
    
    # 1. Inject a 'WAKE' agent (recent heartbeat)
    wake_sid = "wake-123"
    wake_data = {
        "identity": {"name": "ActiveAgent", "rank": "Officer"},
        "last_heartbeat": datetime.now(timezone.utc).isoformat()
    }
    redis_client.hset("koad:state", f"koad:session:{wake_sid}", json.dumps(wake_data))

    # 2. Inject a 'DARK' agent (old heartbeat)
    dark_sid = "dark-456"
    dark_data = {
        "identity": {"name": "IdleAgent", "rank": "Crew"},
        "last_heartbeat": (datetime.now(timezone.utc) - timedelta(minutes=5)).isoformat()
    }
    redis_client.hset("koad:state", f"koad:session:{dark_sid}", json.dumps(dark_data))

    # 3. Run koad crew
    result = spine.run_koad(["crew"])
    assert result.returncode == 0
    assert "ActiveAgent" in result.stdout
    assert "WAKE" in result.stdout
    assert "IdleAgent" in result.stdout
    assert "DARK" in result.stdout
    assert "Total Wake Personnel:" in result.stdout

def test_koad_doctor_full_pass(spine):
    """Verify koad doctor reports PASS when all systems are up."""
    result = spine.run_koad(["doctor"])
    assert result.returncode == 0
    assert "Neural Link & Grid Integrity" in result.stdout
    assert "PASS" in result.stdout
    assert "Engine Room (Redis):" in result.stdout
    assert "Backbone (Spine):" in result.stdout
    assert "Memory Bank (SQLite):" in result.stdout
    assert "Neural Identity:" in result.stdout

def test_preflight_critical_redis_down(koad_env):
    """Verify that a command fails immediately if Redis is down."""
    # koad_env starts redis by default, so we stop it
    koad_env.redis_proc.terminate()
    koad_env.redis_proc.wait()
    
    # Try to run query (which is not excluded from pre-flight)
    result = koad_env.run_koad(["query", "test"])
    assert result.returncode != 0
    assert "CRITICAL" in result.stderr
    assert "KoadOS Kernel is OFFLINE." in result.stderr
    assert "Neural Bus (Redis UDS) is missing" in result.stderr

def test_preflight_degraded_spine_down(koad_env):
    """Verify that a command warns but proceeds if only Spine is down."""
    # Redis is up (default in koad_env), but spine is not started
    # Ensure spine socket doesn't exist
    spine_socket = koad_env.koad_home / "kspine.sock"
    assert not spine_socket.exists()
    
    # Run query
    result = koad_env.run_koad(["query", "test"])
    # Should proceed to query logic (which might find nothing but return 0)
    assert "WARNING" in result.stderr
    assert "KoadOS Kernel is DEGRADED." in result.stderr
    assert "Orchestrator (kspine.sock) is missing" in result.stderr
    # It shouldn't exit with 1
    assert result.returncode == 0

def test_preflight_skip_check_for_doctor(koad_env):
    """Verify that 'doctor' command skips pre-flight check so it can diagnose outages."""
    koad_env.redis_proc.terminate()
    koad_env.redis_proc.wait()
    
    result = koad_env.run_koad(["doctor"])
    # Should not have the [CRITICAL] header from pre-flight
    assert "CRITICAL" not in result.stderr
    assert "KoadOS Kernel is OFFLINE." not in result.stderr
    # But doctor itself should report the failure
    assert "FAIL" in result.stdout
    assert "Engine Room (Redis):" in result.stdout
