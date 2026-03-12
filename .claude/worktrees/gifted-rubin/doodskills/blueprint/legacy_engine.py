#!/usr/bin/env python3
import sys
import os
import json
import argparse
import subprocess
from pathlib import Path

def render_string(template, variables):
    for key, val in variables.items():
        template = template.replace(f"{{{{{key}}}}}", val)
    return template

def main():
    parser = argparse.ArgumentParser(description="KoadOS v3 Blueprint Engine")
    parser.add_argument("action", choices=["list", "use"], help="Action to perform")
    parser.add_argument("blueprint", nargs="?", help="Blueprint ID")
    parser.add_argument("--var", action="append", help="Variables in key=val format")
    args = parser.parse_args()

    home = Path.home() / ".koad-os"
    blueprints_dir = home / "blueprints"

    if args.action == "list":
        print("--- Available Blueprints ---")
        for bp_path in blueprints_dir.iterdir():
            if bp_path.is_dir():
                conf_path = bp_path / "blueprint.json"
                if conf_path.exists():
                    with open(conf_path) as f:
                        conf = json.load(f)
                        print(f"- {conf['id']}: {conf['name']} ({conf['description']})")
        return

    if args.action == "use":
        if not args.blueprint:
            print("Error: No blueprint specified.")
            sys.exit(1)

        bp_path = blueprints_dir / args.blueprint
        conf_path = bp_path / "blueprint.json"
        if not conf_path.exists():
            print(f"Error: Blueprint '{args.blueprint}' not found.")
            sys.exit(1)

        with open(conf_path) as f:
            conf = json.load(f)

        variables = {}
        # Default vars
        variables["project_root"] = os.getcwd()
        
        # Parse CLI vars
        if args.var:
            for v in args.var:
                k, val = v.split("=", 1)
                variables[k] = val

        # Check for missing vars
        for var_name in conf.get("variables", {}):
            if var_name not in variables:
                val = input(f"Enter {var_name} ({conf['variables'][var_name]}): ")
                variables[var_name] = val

        print(f"> Applying blueprint: {conf['name']}...")

        for step in conf.get("steps", []):
            stype = step.get("type")
            if stype == "mkdir":
                path = render_string(step["path"], variables)
                print(f"  [mkdir] {path}")
                os.makedirs(path, exist_ok=True)
            
            elif stype == "render":
                src = bp_path / "files" / step["src"]
                dest = Path(render_string(step["dest"], variables))
                print(f"  [render] {dest}")
                
                with open(src) as f:
                    content = f.read()
                
                rendered = render_string(content, variables)
                os.makedirs(dest.parent, exist_ok=True)
                with open(dest, "w") as f:
                    f.write(rendered)

            elif stype == "run_shell":
                cmd = render_string(step["cmd"], variables)
                print(f"  [shell] {cmd}")
                subprocess.run(cmd, shell=True, check=True)

        print("> Blueprint applied successfully.")

if __name__ == "__main__":
    main()
