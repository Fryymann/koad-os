#!/usr/bin/env python3
import json
import sys
import os

def get_mapping(query=None):
    config_path = os.path.expanduser("~/.koad-os/koad.json")
    try:
        with open(config_path, "r") as f:
            config = json.load(f)
    except Exception as e:
        print(f"Error loading config: {e}")
        return

    mappings = config.get("filesystem", {}).get("mappings", {})
    if query:
        path = mappings.get(query)
        if path:
            print(path)
        else:
            print(f"No mapping found for: {query}")
    else:
        for name, path in mappings.items():
            print(f"{name}: {path}")

if __name__ == "__main__":
    query = sys.argv[1] if len(sys.argv) > 1 else None
    get_mapping(query)
