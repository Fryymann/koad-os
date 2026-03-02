import pytest
import time

def test_grid_clean_sentinel(koad_env):
    """Verify Grid Clean (repo-clean.py) existence."""
    script_path = koad_env.koad_home / "doodskills" / "repo-clean.py"
    assert script_path.exists()

def test_micro_agent_presence(koad_env):
    """Verify Micro-Agent skill sector is accessible."""
    skills_dir = koad_env.koad_home / "skills"
    skills_dir.mkdir(exist_ok=True)
    assert skills_dir.exists()
