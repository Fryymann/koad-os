import pytest
import os
from pathlib import Path

def test_neural_config_sentinel(koad_env):
    """Verify Neural Config (config/kernel.toml) exists and is valid."""
    config_path = koad_env.koad_home / "config/kernel.toml"
    assert config_path.exists()
    assert config_path.stat().st_size > 0

def test_master_memory_sentinel(koad_env):
    """Verify Master Memory (koad.db) is initialized."""
    # koad whoami triggers DB creation
    koad_env.run_koad(["whoami"])
    db_path = koad_env.koad_home / "koad.db"
    assert db_path.exists()

def test_control_signal_sentinel(spine):
    """Verify Control Signal (koad-redis.sock) is active when backbone is energized."""
    sock_path = spine.koad_home / "koad-redis.sock"
    assert sock_path.exists()

def test_uplink_logs_sentinel(koad_env):
    """Verify Uplink Logs (SESSION_LOG.md) presence."""
    # koad whoami should trigger log creation if implemented
    koad_env.run_koad(["whoami"])
    log_path = koad_env.koad_home / "SESSION_LOG.md"
    # Placeholder for sentinel logic
    pass
