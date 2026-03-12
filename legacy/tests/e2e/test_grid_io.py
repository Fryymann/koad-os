import pytest
import requests
import time

def test_gateway_health_uplink(koad_env):
    """Verify the Grid I/O (Gateway) is alive and responding on port 3000."""
    # We need to start the gateway binary
    import subprocess
    import os
    
    my_env = os.environ.copy()
    my_env["KOAD_HOME"] = str(koad_env.koad_home)
    
    # We use the bin/kgateway we confirmed exists
    gateway_proc = subprocess.Popen(
        [str(koad_env.bin_dir / "kgateway")],
        env=my_env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )
    
    try:
        # Wait for gateway to bind to 3000
        connected = False
        for _ in range(100): # More patient
            try:
                # Gateway serves static files from dist at root
                resp = requests.get("http://localhost:3000/", timeout=1)
                # It might return 404 if dist is empty, but that means it's running
                if resp.status_code in [200, 404]:
                    connected = True
                    break
            except:
                time.sleep(0.1)
        
        assert connected, "Grid I/O (Gateway) failed to respond on port 3000"
    finally:
        gateway_proc.terminate()
        gateway_proc.wait()

def test_gateway_spine_bridge(spine):
    """Verify that the Gateway can bridge traffic to the Backbone (Spine)."""
    # This is a placeholder for a more complex integration test 
    # that would verify gRPC-web or WebSocket proxying
    pass
