#!/usr/bin/env python3
import sys
import subprocess
import argparse
from pathlib import Path

def main():
    parser = argparse.ArgumentParser(description="KoadOS Script Validator")
    parser.add_argument("action", choices=["list", "check"])
    parser.add_argument("--path", help="Path to the script to check")
    
    args = parser.parse_args()
    base_dir = Path("/home/ideans/data/skylinks/skylinks-scripts/src")

    if args.action == "list":
        print("Inventory of Skylinks Scripts:")
        for category in ["airtable", "google-apps", "local"]:
            cat_dir = base_dir / category
            if cat_dir.exists():
                print(f"\n[{category.upper()}]")
                for script in cat_dir.rglob("*"):
                    if script.is_file():
                        print(f"- {script.relative_to(base_dir)}")
        return

    if args.action == "check" and args.path:
        script_path = base_dir.parent / args.path
        if not script_path.exists():
            print(f"Error: {script_path} not found.")
            sys.exit(1)
        
        print(f"Checking {script_path.suffix} script: {args.path}")
        if script_path.suffix == ".js":
            subprocess.run(["node", "--check", str(script_path)])
        elif script_path.suffix == ".py":
            subprocess.run(["python3", "-m", "py_compile", str(script_path)])
        elif script_path.suffix == ".ps1":
            print("Note: PowerShell syntax check requires Windows environment.")
        else:
            print(f"No validator for {script_path.suffix}")

if __name__ == "__main__":
    main()
