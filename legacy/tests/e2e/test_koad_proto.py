import pytest

def test_proto_reflection(koad_env):
    """Verify that the system can handle protobuf-defined CLI commands (query)."""
    # Query is a simple way to test the CLI's handling of SQLite/Protobuf structures
    result = koad_env.run_koad(["query", "nonexistent_term"])
    assert result.returncode == 0
    # Should return empty result but not crash
    assert "ID:" not in result.stdout
