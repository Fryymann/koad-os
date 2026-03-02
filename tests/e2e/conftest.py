import os
import shutil
import subprocess
import time
import pytest
import sqlite3
import redis
from pathlib import Path

class KoadTestEnvironment:
    def __init__(self, root_dir: Path):
        self.root_dir = root_dir
        self.koad_home = root_dir / ".koad-os"
        self.bin_dir = self.koad_home / "bin"
        self.socket_path = self.koad_home / "koad.sock"
        self.db_path = self.koad_home / "koad.db"
        self.redis_proc = None
        self.spine_proc = None
        self.spine_log_handle = None

    def setup(self, source_root: Path):
        self.koad_home.mkdir(parents=True, exist_ok=True)
        self.bin_dir.mkdir(parents=True, exist_ok=True)
        
        debug_bins = source_root / "target" / "debug"
        bin_mapping = {
            "koad": "koad",
            "koad-spine": "kspine",
            "koad-cli": "koad-cli"
        }
        for src_name, dest_name in bin_mapping.items():
            src = debug_bins / src_name
            if src.exists():
                shutil.copy(src, self.bin_dir / dest_name)

        # Copy doodskills for KCM
        dest_skills = self.koad_home / "doodskills"
        dest_skills.mkdir(parents=True, exist_ok=True)
        src_skills = source_root / "doodskills"
        if src_skills.exists():
            for item in src_skills.iterdir():
                if item.is_file():
                    shutil.copy(item, dest_skills / item.name)

        # Symlink venv so KCM can find python with rich
        venv_src = source_root / "venv"
        venv_dest = self.koad_home / "venv"
        if venv_src.exists():
            os.symlink(venv_src, venv_dest)

        koad_json_content = """
{
  "version": "3.2",
  "identity": {"name": "TestKoad", "role": "Admin", "bio": "E2E Test Identity"},
  "preferences": {
    "languages": ["Rust"],
    "booster_enabled": false,
    "style": "programmatic-first",
    "principles": []
  },
  "drivers": {
    "gemini": {
      "bootstrap": "",
      "mcp_enabled": false,
      "tools": []
    }
  },
  "filesystem": {"mappings": {}, "workspace_symlink": "/tmp/koad_test_data"}
}
"""
        (self.koad_home / "koad.json").write_text(koad_json_content)

    def start_redis(self):
        self.redis_proc = subprocess.Popen(
            ["redis-server", "--port", "0", "--unixsocket", str(self.socket_path)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
        for _ in range(50):
            if self.socket_path.exists():
                break
            time.sleep(0.1)
        else:
            raise RuntimeError("Redis failed to start")

    def start_spine(self):
        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        
        log_path = self.koad_home / "kspine.log"
        self.spine_log_handle = open(log_path, "w")
        
        self.spine_proc = subprocess.Popen(
            [str(self.bin_dir / "kspine")],
            stdout=self.spine_log_handle,
            stderr=self.spine_log_handle,
            env=my_env,
            text=True
        )
        spine_socket = self.koad_home / "kspine.sock"
        for _ in range(200):
            if spine_socket.exists():
                break
            time.sleep(0.1)
        else:
            if self.spine_proc.poll() is not None:
                raise RuntimeError(f"kspine crashed. See {log_path}")
            raise RuntimeError(f"kspine failed to start at {spine_socket}")

    def stop(self):
        if self.spine_proc:
            self.spine_proc.terminate()
            try:
                self.spine_proc.wait(timeout=5)
            except:
                self.spine_proc.kill()
        if self.spine_log_handle:
            self.spine_log_handle.close()
            
        if self.redis_proc:
            self.redis_proc.terminate()
            try:
                self.redis_proc.wait(timeout=5)
            except:
                self.redis_proc.kill()

    def run_koad(self, args, env=None):
        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        if env:
            my_env.update(env)
        cmd = [str(self.bin_dir / "koad")] + args
        return subprocess.run(cmd, capture_output=True, text=True, env=my_env)

    def run_cli(self, args):
        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        cmd = [str(self.bin_dir / "koad-cli")] + args
        return subprocess.run(cmd, capture_output=True, text=True, env=my_env)

@pytest.fixture
def koad_env(tmp_path):
    source_root = Path("/home/ideans/.koad-os")
    env = KoadTestEnvironment(tmp_path)
    env.setup(source_root)
    env.start_redis()
    yield env
    env.stop()

@pytest.fixture
def spine(koad_env):
    koad_env.start_spine()
    return koad_env

@pytest.fixture
def redis_client(koad_env):
    return redis.Redis(unix_socket_path=str(koad_env.socket_path), decode_responses=True)

@pytest.fixture
def db_conn(koad_env):
    # koad remember creates the DB if missing
    koad_env.run_koad(["whoami"]) 
    conn = sqlite3.connect(koad_env.db_path)
    yield conn
    conn.close()
