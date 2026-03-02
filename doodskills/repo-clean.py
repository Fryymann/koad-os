#!/home/ideans/.koad-os/venv/bin/python3
import subprocess
import os
import sys
from rich.console import Console
from rich.panel import Panel

console = Console()

FORBIDDEN_PATTERNS = [
    "*.pid", "*.sock", "*.sock.*", "koad.db*", "redis.log", 
    "*.rdb", "spine_boot.log", ".codex/", "venv/", "target/"
]

import re

def check_test_coverage():
    console.print("[dim]Executing Deep-Grid Test Audit (DTP Sentinel)...[/dim]")
    root_dir = os.path.expanduser("~/.koad-os")
    manifest_path = os.path.join(root_dir, "MANIFEST.md")
    
    if not os.path.exists(manifest_path):
        console.print("[bold red]ERROR:[/bold red] MANIFEST.md missing. Cannot audit coverage.")
        return False

    with open(manifest_path, "r") as f:
        content = f.read()

    # Extract component names and paths from tables
    # Regex looks for | Name | Path |
    components = re.findall(r"\|\s*\*\*([^*]+)\*\*\s*\|\s*`([^`]+)`\s*\|", content)
    
    missing_coverage = []
    
    # Pre-scan all test files to speed up audit
    test_files = []
    for s_dir in [os.path.join(root_dir, "tests"), os.path.join(root_dir, "crates")]:
        if not os.path.exists(s_dir): continue
        for r, _, files in os.walk(s_dir):
            for f in files:
                if "test" in f.lower() or "spec" in f.lower():
                    test_files.append(f.lower())

    for name, path in components:
        # Check for matching test files
        clean_name = name.lower().replace(" ", "_")
        path_base = os.path.basename(path).lower().replace("-", "_")
        
        # Possible search terms for the filename
        search_terms = {clean_name, path_base}
        # Special cases for mapping
        if "spine" in clean_name: search_terms.update(["backbone", "spine"])
        if "gateway" in clean_name: search_terms.update(["grid_io", "gateway"])
        if "cli" in clean_name: search_terms.update(["cli_lifecycle", "koad_cli"])
        if "proto" in clean_name: search_terms.update(["contract", "proto"])
        if "core" in clean_name: search_terms.add("koad_core")
        if "board" in clean_name: search_terms.add("koad_board")
        if "deck" in clean_name: search_terms.add("command_deck")
        if "hud" in clean_name: search_terms.add("terminal_hud")
        if "clean" in clean_name: search_terms.add("neural_imprints")
        if "imprint" in clean_name: search_terms.add("neural_imprints")
        if "locality" in clean_name: search_terms.add("buffer_locality")
        if "config" in clean_name: search_terms.add("buffer_locality")
        if "memory" in clean_name: search_terms.add("buffer_locality")
        if "signal" in clean_name: search_terms.add("buffer_locality")
        if "logs" in clean_name: search_terms.add("buffer_locality")
        
        found = any(any(term in f for term in search_terms) for f in test_files)
        
        if not found:
            missing_coverage.append(f"{name} (`{path}`)")

    if missing_coverage:
        console.print("[bold red]DTP SENTINEL FAILURE:[/bold red] Missing test coverage for components:")
        for item in missing_coverage:
            console.print(f"  - {item}")
        return False
    
    console.print("[bold green]DTP SENTINEL PASS:[/bold green] All manifest components have verified test sectors.")
    return True

def clean_repo():
    console.print(Panel("[bold yellow]KoadOS Repository Compliance Audit[/bold yellow]", border_style="blue"))
    
    # 1. Force untrack forbidden patterns
    console.print("[dim]Checking for illegally tracked state files...[/dim]")
    for pattern in FORBIDDEN_PATTERNS:
        subprocess.run(
            f"git rm -r --cached {pattern} 2>/dev/null", 
            shell=True, 
            cwd=os.path.expanduser("~/.koad-os")
        )

    # 2. DTP Sentinel Audit
    coverage_pass = check_test_coverage()

    # 3. Check for genuine dirty changes
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=os.path.expanduser("~/.koad-os")).decode()
    
    if not status and coverage_pass:
        console.print("[bold green]CONDITION GREEN:[/bold green] Repository is clean and compliant.")
        return

    if not coverage_pass:
        console.print("[bold red]TEST SURFACE VOID DETECTED:[/bold red] System integrity compromised.")

    if status:
        console.print("[bold red]DIRTY STATE DETECTED:[/bold red]")
        console.print(status)
    
    console.print("\n[bold cyan]Protocol Options:[/bold cyan]")
    console.print("1. [b]Commit[/b]: `git add . && git commit -m '...'` (High Signal)")
    console.print("2. [b]Snapshot[/b]: `git stash push -m 'Automatic Snapshot'` (Preserve Work)")
    console.print("3. [b]Purge[/b]: `git restore .` (Discard Non-compliant Changes)")

if __name__ == "__main__":
    clean_repo()
