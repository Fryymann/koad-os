#!/usr/bin/env python3
import sys
import subprocess
import json

def main():
    print("KoadOS Skill: Hello Koad")
    print("Arguments received:", sys.argv[1:])
    
    # Example of a skill calling the koad CLI back
    try:
        result = subprocess.run(
            ["/home/ideans/.koad-os/core/rust/target/release/koad", "auth"],
            capture_output=True, text=True, check=True
        )
        print("Koad CLI Auth Response:")
        print(result.stdout.strip())
    except Exception as e:
        print(f"Failed to call koad CLI: {e}")

if __name__ == "__main__":
    main()
