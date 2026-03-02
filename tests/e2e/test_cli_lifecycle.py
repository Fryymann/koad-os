import pytest
import json

def test_koad_boot(koad_env):
    """Verify that koad boot runs and generates a valid context."""
    result = koad_env.run_koad(["boot", "--agent", "TestAgent"])
    assert result.returncode == 0
    assert "Identity: TestKoad (Admin)" in result.stdout
    assert "Session:" in result.stdout

def test_koad_whoami(koad_env):
    """Verify the whoami command reports the correct identity from koad.json."""
    result = koad_env.run_koad(["whoami"])
    assert result.returncode == 0
    assert "TestKoad" in result.stdout
    assert "Admin" in result.stdout

def test_agent_hydration_on_boot(spine):
    """Verify that koad boot initializes a session and receives a briefing."""
    result = spine.run_koad(["boot", "--agent", "TestAgent"])
    assert result.returncode == 0
    assert "Identity: TestKoad (Admin)" in result.stdout
    # Briefing should be present if spine hydrated it
    assert "MISSION BRIEFING" in result.stdout
    assert "Welcome, Agent" in result.stdout

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
    assert "Fetching Project Board status" in result.stdout

def test_koad_stat_json(spine, redis_client):
    """Verify that koad stat --json returns raw JSON."""
    stats = {"cpu_usage": 10.0, "memory_usage": 500, "skill_count": 1, "active_tasks": 0}
    redis_client.hset("koad:state", "system_stats", json.dumps(stats))
    
    result = spine.run_koad(["stat", "--json"])
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert data["cpu_usage"] == 10.0
