#!/usr/bin/env python3
import sys
import subprocess
import os

def run_koad(args):
    try:
        subprocess.run(["/home/ideans/.koad-os/bin/koad"] + args, check=True)
        return True
    except:
        return False

def maintain_continuity(finding=None):
    """
    Automates the transition between research and execution.
    1. Harvests the immediate findings into the knowledge base.
    2. Runs a surgical query to pull relevant previous context.
    """
    if finding:
        print(f"--- Koad Continuity: Harvesting Finding ---")
        run_koad(["remember", "fact", finding, "--tags", "continuity,gemini-session"])
    
    # Identify relevant tags from CWD
    tags = []
    if os.path.exists("package.json"): tags.append("node")
    if os.path.exists("Cargo.toml"): tags.append("rust")
    
    if tags:
        print(f"--- Koad Continuity: Injecting Related Context ({','.join(tags)}) ---")
        for tag in tags:
            run_koad(["query", tag])
            
    return True

if __name__ == "__main__":
    finding_arg = sys.argv[1] if len(sys.argv) > 1 else None
    maintain_continuity(finding_arg)
