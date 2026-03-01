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

    # 2. Check for genuine dirty changes
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=os.path.expanduser("~/.koad-os")).decode()
    
    if not status:
        console.print("[bold green]CONDITION GREEN:[/bold green] Repository is clean and compliant.")
        return

    # 3. Handle remaining dirt
    console.print("[bold red]DIRTY STATE DETECTED:[/bold red]")
    console.print(status)
    
    console.print("\n[bold cyan]Protocol Options:[/bold cyan]")
    console.print("1. [b]Commit[/b]: `git add . && git commit -m '...'` (High Signal)")
    console.print("2. [b]Snapshot[/b]: `git stash push -m 'Automatic Snapshot'` (Preserve Work)")
    console.print("3. [b]Purge[/b]: `git restore .` (Discard Non-compliant Changes)")

if __name__ == "__main__":
    clean_repo()
