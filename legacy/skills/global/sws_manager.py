#!/usr/bin/env python3
import sys
import subprocess
import argparse
import os
import json
import shutil
from pathlib import Path

# Config
GCP_PROJECT = os.getenv("GCP_PROJECT", "default-project")
REGION = os.getenv("GCP_REGION", "us-central1")
DEFAULT_SWS_DIR = "/mnt/c/data/skylinks/functions/src"
CANONICAL_SOURCE = "sws-sgc-registration"
TEMPLATE_DIR = Path(os.path.expanduser("~/.koad-os/templates/skylinks_sws"))

def run_cmd(cmd, cwd=None, shell=False):
    print(f"Executing: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    result = subprocess.run(cmd, cwd=cwd, shell=shell, text=True)
    if result.returncode != 0:
        print(f"Command failed with code {result.returncode}")
        return False
    return True

def render_template(template_path, variables):
    with open(template_path, "r") as f:
        content = f.read()
    for key, value in variables.items():
        content = content.replace(f"{{{{ {key} }}}}", value)
    return content

def analyze_and_scaffold_team(func_dir):
    print("\n--- Running Project Team Analysis ---")
    analyzer_path = Path(os.path.expanduser("~/.koad-os/skills/global/project_analyzer.py"))
    if not analyzer_path.exists():
        print("Warning: project_analyzer.py skill not found. Skipping team design.")
        return

    # We run the analyzer and capture output
    result = subprocess.run([sys.executable, str(analyzer_path), str(func_dir)], capture_output=True, text=True)
    if result.returncode == 0:
        analysis_output = result.stdout
        print(analysis_output)
        
        team_design_path = func_dir / "docs/ANTIGRAVITY/TEAM_DESIGN.md"
        with open(team_design_path, "w") as f:
            f.write(f"# Project Team Design - {func_dir.name}\n\n")
            f.write("## Automated Analysis Findings\n")
            f.write("```\n")
            f.write(analysis_output)
            f.write("```\n\n")
            f.write("## Role Assignment Strategy\n")
            f.write("- **PM**: Orchestration & Spec management.\n")
            f.write("- **Dev-Backend**: Node.js/GCP Logic.\n")
            f.write("- **Dev-Frontend**: WordPress/Integration Logic.\n")
            f.write("- **QA**: Testing & Validation.\n")
        print(f"Created: {team_design_path}")

def action_init(args):
    if not args.name:
        print("Error: --name is required for init")
        sys.exit(1)

    base_dir = Path(os.getenv("SWS_FUNCTIONS_DIR", DEFAULT_SWS_DIR))
    func_dir = base_dir / args.name
    source_dir = base_dir / args.source if args.source else base_dir / CANONICAL_SOURCE

    if func_dir.exists() and any(func_dir.iterdir()) and not args.force:
        print(f"Error: Directory {func_dir} already exists and is not empty. Use --force to override.")
        sys.exit(1)

    print(f"Initializing new SWS project: {args.name} in {func_dir}")
    func_dir.mkdir(parents=True, exist_ok=True)

    # Variables for templating
    title = args.name.replace("sws-", "").replace("-", " ").title()
    camel_name = "".join(x.capitalize() for x in args.name.replace("sws-", "").split("-"))
    handler_name = f"handle{camel_name}"
    
    variables = {
        "name": args.name,
        "title": title,
        "handler_name": handler_name
    }

    # Template Rendering
    if TEMPLATE_DIR.exists():
        print(f"Rendering templates from {TEMPLATE_DIR}...")
        for root, dirs, files in os.walk(TEMPLATE_DIR):
            rel_path = Path(root).relative_to(TEMPLATE_DIR)
            target_path = func_dir / rel_path
            target_path.mkdir(parents=True, exist_ok=True)
            
            for file in files:
                if file.endswith(".j2"):
                    src_file = Path(root) / file
                    dest_file_name = file.replace(".j2", "")
                    dest_file = target_path / dest_file_name
                    content = render_template(src_file, variables)
                    with open(dest_file, "w") as f:
                        f.write(content)
                    print(f"  Rendered: {rel_path / dest_file_name}")
                else:
                    shutil.copy2(Path(root) / file, target_path / file)
                    print(f"  Copied: {rel_path / file}")

    # Inject Canonical Utilities
    if source_dir.exists():
        print(f"Injecting standard utilities from {source_dir}...")
        utils = ["config.js", "logger.js", "validators.js", "tokenService.js", "rateLimiter.js"]
        src_backend = source_dir / "src/backend/gcp"
        dest_backend = func_dir / "src/backend/gcp"
        for util in utils:
            src_file = src_backend / util
            if src_file.exists():
                shutil.copy2(src_file, dest_backend / util)
                print(f"  Copied: {util}")

    # Empty docs
    for doc in ["DELEGATION_BRIEF.md", "HANDOFF_PACKET.md", "KICKOFF_TASKS.md", "SPEC_REVIEW.md", "WORK_LOG.md"]:
        doc_path = func_dir / f"docs/ANTIGRAVITY/{doc}"
        if not doc_path.exists():
            with open(doc_path, "w") as f:
                f.write(f"# {doc.replace('.md', '').replace('_', ' ').title()}\n")
            print(f"  Created empty doc: {doc}")

    # Run Team Analysis
    analyze_and_scaffold_team(func_dir)

    # Git
    if run_cmd(["git", "init"], cwd=func_dir):
        run_cmd(["git", "add", "."], cwd=func_dir)
        run_cmd(["git", "commit", "-m", f"Initial commit: {args.name} scaffolded with team analysis"], cwd=func_dir)

    # Dispatch heavy task
    print(f"\nDispatching background npm install for {args.name}...")
    dispatch_cmd = f"npm --prefix {func_dir}/src/backend/gcp install"
    run_cmd(["koad", "dispatch", dispatch_cmd])

    # Scan project
    run_cmd(["koad", "scan", str(func_dir)])

def main():
    parser = argparse.ArgumentParser(description="KoadOS SWS Deployment Dispatcher")
    subparsers = parser.add_subparsers(dest="action", required=True)
    parser_init = subparsers.add_parser("init", help="Initialize new SWS project")
    parser_init.add_argument("--name", required=True, help="New function name")
    parser_init.add_argument("--source", help="Source project to copy utilities from")
    parser_init.add_argument("--force", action="store_true", help="Force overwrite existing directory")
    # ... other parsers
    args = parser.parse_args()
    if args.action == "init":
        action_init(args)

if __name__ == "__main__":
    main()
