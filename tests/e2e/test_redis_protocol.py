import redis
import os
from pathlib import Path

def test_redis_basic_connectivity():
    socket_path = "/tmp/koad_protocol_test.sock"
    import subprocess
    import time
    
    # Start a clean redis server
    proc = subprocess.Popen(["redis-server", "--port", "0", "--unixsocket", socket_path], stdout=subprocess.DEVNULL)
    time.sleep(0.5)
    
    try:
        client = redis.Redis(unix_socket_path=socket_path, decode_responses=True)
        client.set("test_key", "test_val")
        assert client.get("test_key") == "test_val"
        print("Python redis-py (RESP2/3 auto) -> PASS")
    finally:
        proc.terminate()
        if os.path.exists(socket_path): os.remove(socket_path)

if __name__ == "__main__":
    test_redis_basic_connectivity()
