#!/usr/bin/env python3
import os
import sys
import re
from pathlib import Path

def bump_version(new_version):
    home = Path(os.environ.get("KOAD_HOME", Path.home() / ".koad-os"))
    print(f"Bumping KoadOS to v{new_version} in {home}...")

    # 1. Cargo.toml (Workspace)
    cargo_path = home / "Cargo.toml"
    if cargo_path.exists():
        content = cargo_path.read_text()
        new_content = re.sub(r'version = "\d+\.\d+\.\d+(-[a-z]+)?"', f'version = "{new_version}"', content, count=1)
        cargo_path.write_text(new_content)
        print(f"[OK] Updated {cargo_path}")

    # 2. koad.json (Schema)
    # Extract major.minor for schema
    schema_version = ".".join(new_version.split(".")[:2])
    koad_json = home / "koad.json"
    if koad_json.exists():
        content = koad_json.read_text()
        new_content = re.sub(r'"version": "\d+\.\d+"', f'"version": "{schema_version}"', content)
        koad_json.write_text(new_content)
        print(f"[OK] Updated {koad_json} (Schema: {schema_version})")

    # 3. Documentation Headers
    docs = ["README.md", "SPEC.md", "ARCHITECTURE.md", "AGENT_INSTALL.md"]
    for doc in docs:
        doc_path = home / doc
        if doc_path.exists():
            content = doc_path.read_text()
            # Update headers like "KoadOS v3.0"
            new_content = re.sub(r'KoadOS v\d+\.\d+', f'KoadOS v{schema_version}', content)
            # Update specific version notes
            new_content = re.sub(r'version: "\d+\.\d+"', f'version: "{schema_version}"', new_content)
            doc_path.write_text(new_content)
            print(f"[OK] Updated {doc_path}")

    # 4. internal version reporting (spine rpc)
    rpc_path = home / "crates/koad-spine/src/rpc/mod.rs"
    if rpc_path.exists():
        content = rpc_path.read_text()
        new_content = re.sub(r'version: "\d+\.\d+\.\d+(-[a-z]+)?"', f'version: "{new_version}"', content)
        rpc_path.write_text(new_content)
        print(f"[OK] Updated {rpc_path}")

    print("
Version bump complete. Remember to run 'koad publish' if everything looks correct.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: version-bump.py <new_version> (e.g. 3.1.0)")
        sys.exit(1)
    
    bump_version(sys.argv[1])
