import os
import shutil
import subprocess
import time
import pytest
import sqlite3
import redis
import signal
import socket
import json
from pathlib import Path

class KoadTestEnvironment:
    def __init__(self, root_dir: Path):
        self.root_dir = root_dir
        self.koad_home = root_dir / ".koad-os"
        self.bin_dir = self.koad_home / "bin"
        self.redis_socket = self.koad_home / "koad.sock"
        self.spine_socket = self.koad_home / "kspine.sock"
        self.db_path = self.koad_home / "koad.db"
        self.spine_grpc_addr = "http://127.0.0.1:50051" # Default
        self.redis_proc = None
        self.spine_proc = None
        self.spine_log_handle = None

    def setup(self, source_root: Path):
        self.zombie_sweep()
        self.koad_home.mkdir(parents=True, exist_ok=True)
        self.bin_dir.mkdir(parents=True, exist_ok=True)
        
        # 1. Deploy Binaries (Parity check)
        release_bins = source_root / "target" / "release"
        bin_mapping = {
            "koad": "koad", 
            "koad-spine": "kspine",
            "koad-gateway": "kgateway"
        }
        for src_name, dest_name in bin_mapping.items():
            src = release_bins / src_name
            if src.exists():
                shutil.copy(src, self.bin_dir / dest_name)

        # 2. Derive Config from CLI (True Parity)
        env = os.environ.copy()
        env["KOAD_HOME"] = str(self.koad_home)
        res = subprocess.run([str(self.bin_dir / "koad"), "config", "--json"], capture_output=True, text=True, env=env)
        if res.returncode == 0:
            cfg = json.loads(res.stdout)
            self.redis_socket = Path(cfg["redis_socket"])
            self.spine_socket = Path(cfg["spine_socket"])
            self.db_path = Path(cfg["db_path"])
        else:
            print(f"WARN: koad config failed ({res.stderr}). Using internal defaults.")

        # 3. Setup Context
        dest_skills = self.koad_home / "doodskills"
        dest_skills.mkdir(parents=True, exist_ok=True)
        src_skills = source_root / "doodskills"
        if src_skills.exists():
            for item in src_skills.iterdir():
                if item.is_file():
                    shutil.copy(item, dest_skills / item.name)

        # 4. Trigger Schema Creation via CLI
        # Note: We pass no_check to avoid Spine dependencies during setup
        self.run_koad(["--no-check", "whoami"])

        # 5. Populate Identities
        self.setup_identities()

    def setup_identities(self):
        """Insert test identities using established schema."""
        conn = sqlite3.connect(self.db_path)
        c = conn.cursor()
        
        identities = [
            ('TestAgent', 'TestAgent', 'E2E Test Agent', 3),
            ('TestKoad', 'TestKoad', 'E2E Test Koad', 1),
            ('Koad', 'Koad', 'Principal Koad OS Identity', 1),
            ('Vigil', 'Vigil', 'E2E Test Auditor', 2),
            ('Pippin', 'Pippin', 'E2E Test PM', 3)
        ]
        
        now = "2026-03-03T00:00:00"
        for id, name, bio, tier in identities:
            c.execute("INSERT OR REPLACE INTO identities (id, name, bio, tier, created_at) VALUES (?, ?, ?, ?, ?)", (id, name, bio, tier, now))
            role = 'pm' if id == 'Pippin' else 'admin'
            c.execute("INSERT OR REPLACE INTO identity_roles (identity_id, role) VALUES (?, ?)", (id, role))
        
        conn.commit()
        conn.close()

    def zombie_sweep(self):
        try:
            subprocess.run(["pkill", "-9", "-f", str(self.bin_dir / "kspine")], stderr=subprocess.DEVNULL)
            subprocess.run(["pkill", "-9", "-f", f"redis-server.*{self.redis_socket}"], stderr=subprocess.DEVNULL)
        except:
            pass

    def start_redis(self):
        self.redis_proc = subprocess.Popen(
            ["redis-server", "--port", "0", "--unixsocket", str(self.redis_socket)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            preexec_fn=os.setsid 
        )
        for _ in range(50):
            if self.redis_socket.exists():
                break
            time.sleep(0.1)
        else:
            raise RuntimeError("Redis failed to start")

    def start_spine(self):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind(('', 0))
            self.spine_port = s.getsockname()[1]

        self.spine_grpc_addr = f"http://127.0.0.1:{self.spine_port}"
        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        my_env["SPINE_GRPC_ADDR"] = self.spine_grpc_addr
        my_env["REDIS_SOCKET"] = str(self.redis_socket)
        my_env["SPINE_SOCKET"] = str(self.spine_socket)

        log_path = self.koad_home / "kspine.log"
        self.spine_log_handle = open(log_path, "w")

        self.spine_proc = subprocess.Popen(
            [str(self.bin_dir / "kspine")],
            stdout=self.spine_log_handle,
            stderr=self.spine_log_handle,
            env=my_env,
            text=True,
            preexec_fn=os.setsid
        )
        for _ in range(200):
            if self.spine_socket.exists():
                break
            time.sleep(0.1)
        else:
            raise RuntimeError(f"kspine failed to start at {self.spine_socket}")

    def stop(self):
        if self.spine_proc:
            try: os.killpg(os.getpgid(self.spine_proc.pid), signal.SIGKILL)
            except: pass
            self.spine_proc.wait()
        if self.redis_proc:
            try: os.killpg(os.getpgid(self.redis_proc.pid), signal.SIGKILL)
            except: pass
            self.redis_proc.wait()

    def run_koad(self, args, env=None):
        my_env = os.environ.copy()
        my_env.pop("GEMINI_CLI", None)
        my_env.pop("CODEX_CLI", None)
        my_env["KOAD_HOME"] = str(self.koad_home)
        my_env["REDIS_SOCKET"] = str(self.redis_socket)
        my_env["SPINE_SOCKET"] = str(self.spine_socket)
        if hasattr(self, 'spine_grpc_addr'):
            my_env["SPINE_GRPC_ADDR"] = str(self.spine_grpc_addr)
        
        if env: my_env.update(env)
        cmd = [str(self.bin_dir / "koad")] + args
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
def db_conn(koad_env):
    conn = sqlite3.connect(koad_env.db_path)
    yield conn
    conn.close()
