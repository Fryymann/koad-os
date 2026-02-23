#!/usr/bin/env python3
import sys
import os
import json
import subprocess
from pathlib import Path

def run_koad_command(args):
    """Helper to run koad CLI commands."""
    try:
        subprocess.run(["koad"] + args, check=True, capture_output=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False

def scan_project(path="."):
    """Scans for koad project configuration."""
    cwd = Path(path).resolve()
    koad_dir = cwd / ".koad"
    
    if not koad_dir.exists():
        return {"status": "no_koad_project", "path": str(cwd)}

    project_file = koad_dir / "project/koad_project.py"
    if not project_file.exists():
        result = {"status": "koad_structure_missing", "path": str(cwd)}
        # Remember this as a learning
        run_koad_command(["remember", "fact", f"Scanned {cwd}: Missing koad_project.py structure.", "--tags", "scan,error"])
        return result

    # If found, report it to Koad memory
    run_koad_command(["remember", "fact", f"Discovered Koad project at {cwd}", "--tags", "scan,success"])
    return {"status": "found", "path": str(cwd), "has_core": True}

if __name__ == "__main__":
    # Scan current directory by default or provided path
    target_path = sys.argv[1] if len(sys.argv) > 1 else "."
    result = scan_project(target_path)
    print(json.dumps(result, indent=2))
