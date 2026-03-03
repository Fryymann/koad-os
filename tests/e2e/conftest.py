import os
import shutil
import subprocess
import time
import pytest
import sqlite3
import redis
import signal
import socket
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
        self.zombie_sweep()
        self.koad_home.mkdir(parents=True, exist_ok=True)
        self.bin_dir.mkdir(parents=True, exist_ok=True)
        
        release_bins = source_root / "target" / "release"
        debug_bins = source_root / "target" / "debug"
        bin_mapping = {
            "koad": "koad",
            "koad-spine": "kspine",
            "koad-cli": "koad-cli",
            "koad-gateway": "kgateway",
            "koad-tui": "kdash"
        }
        for src_name, dest_name in bin_mapping.items():
            # Try release first, then debug, then root bin
            src = release_bins / src_name
            if not src.exists():
                src = debug_bins / src_name
            
            if src.exists():
                shutil.copy(src, self.bin_dir / dest_name)
            else:
                # Fallback to bin/ if not in target/
                alt_src = source_root / "bin" / src_name
                if alt_src.exists():
                    shutil.copy(alt_src, self.bin_dir / dest_name)

        # Copy doodskills for KCM
        dest_skills = self.koad_home / "doodskills"
        dest_skills.mkdir(parents=True, exist_ok=True)
        src_skills = source_root / "doodskills"
        if src_skills.exists():
            for item in src_skills.iterdir():
                if item.is_file():
                    shutil.copy(item, dest_skills / item.name)

        # Symlink venv and web
        venv_src = source_root / "venv"
        venv_dest = self.koad_home / "venv"
        if venv_src.exists():
            os.symlink(venv_src, venv_dest)
            
        web_src = source_root / "web"
        web_dest = self.koad_home / "web"
        if web_src.exists():
            os.symlink(web_src, web_dest)

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

    def zombie_sweep(self):
        """Kill any lingering kspine or redis-server processes from previous failed runs."""
        # Simple sweep using pkill for the specific test binary names
        # Note: This might kill non-test processes if they share the same name, 
        # but in our E2E env we use 'kspine' specifically for the test binary.
        try:
            subprocess.run(["pkill", "-9", "-f", str(self.bin_dir / "kspine")], stderr=subprocess.DEVNULL)
            # Only kill redis if it's pointing to our specific test socket
            subprocess.run(["pkill", "-9", "-f", f"redis-server.*{self.socket_path}"], stderr=subprocess.DEVNULL)
        except:
            pass

    def start_redis(self):
        self.redis_proc = subprocess.Popen(
            ["redis-server", "--port", "0", "--unixsocket", str(self.socket_path)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            preexec_fn=os.setsid # Start in new process group
        )
        for _ in range(50):
            if self.socket_path.exists():
                break
            time.sleep(0.1)
        else:
            raise RuntimeError("Redis failed to start")

    def start_spine(self):
        # Find a free port for gRPC
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind(('', 0))
            self.spine_port = s.getsockname()[1]

        self.spine_grpc_addr = f"http://127.0.0.1:{self.spine_port}"

        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        my_env["SPINE_GRPC_ADDR"] = self.spine_grpc_addr

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
        spine_socket = self.koad_home / "kspine.sock"
        for _ in range(200):
            if spine_socket.exists():
                break
            time.sleep(0.1)
        else:
            if self.spine_proc.poll() is not None:
                raise RuntimeError(f"kspine crashed. See {log_path}")
            raise RuntimeError(f"kspine failed to start at {spine_socket}")

    def start_gateway(self):
        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        my_env["SPINE_GRPC_ADDR"] = self.spine_grpc_addr

        
        # Disable GitHub sync for tests unless needed
        my_env["GITHUB_ADMIN_PAT"] = "test_token"
        
        # Find a free port for Gateway
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind(('', 0))
            self.gateway_port = s.getsockname()[1]

        log_path = self.koad_home / "kgateway.log"
        self.gateway_log_handle = open(log_path, "w")
        
        self.gateway_proc = subprocess.Popen(
            [str(self.bin_dir / "kgateway"), "--addr", f"127.0.0.1:{self.gateway_port}"],
            stdout=self.gateway_log_handle,
            stderr=self.gateway_log_handle,
            env=my_env,
            text=True,
            preexec_fn=os.setsid
        )
        # Wait for port
        for _ in range(100):
            try:
                with socket.create_connection(("127.0.0.1", self.gateway_port), timeout=0.1):
                    break
            except:
                time.sleep(0.1)
        else:
            raise RuntimeError(f"kgateway failed to start on port {self.gateway_port}. See {log_path}")

    def stop(self):
        if hasattr(self, 'gateway_proc') and self.gateway_proc:
            try: os.killpg(os.getpgid(self.gateway_proc.pid), signal.SIGKILL)
            except: pass
            self.gateway_proc.wait()
        
        if hasattr(self, 'gateway_log_handle') and self.gateway_log_handle:
            self.gateway_log_handle.close()

        if self.spine_proc:
            try:
                os.killpg(os.getpgid(self.spine_proc.pid), signal.SIGKILL)
            except:
                pass
            self.spine_proc.wait()
            
        if self.spine_log_handle:
            self.spine_log_handle.close()
            
        if self.redis_proc:
            try:
                os.killpg(os.getpgid(self.redis_proc.pid), signal.SIGKILL)
            except:
                pass
            self.redis_proc.wait()

    def run_koad(self, args, env=None):
        my_env = os.environ.copy()
        my_env["KOAD_HOME"] = str(self.koad_home)
        if env: my_env.update(env)
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
def gateway(koad_env):
    koad_env.start_spine()
    koad_env.start_gateway()
    return koad_env

@pytest.fixture
def redis_client(koad_env):
    return redis.Redis(unix_socket_path=str(koad_env.socket_path), decode_responses=True)

@pytest.fixture
def db_conn(koad_env):
    koad_env.run_koad(["whoami"]) 
    conn = sqlite3.connect(koad_env.db_path)
    yield conn
    conn.close()
