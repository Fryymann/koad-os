import pytest

def test_koad_boot(koad_env):
    """Verify that koad boot runs and generates a valid context."""
    result = koad_env.run_koad(["boot"])
    assert result.returncode == 0
    assert "Identity: TestKoad (Admin)" in result.stdout
    assert "Session:" in result.stdout

def test_koad_whoami(koad_env):
    """Verify the whoami command reports the correct identity from koad.json."""
    result = koad_env.run_koad(["whoami"])
    assert result.returncode == 0
    assert "TestKoad" in result.stdout
    assert "Admin" in result.stdout

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
