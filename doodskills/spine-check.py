#!/home/ideans/.koad-os/venv/bin/python3
import redis
import socket
import json
import os
import sys
import time
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

# Configuration
KOAD_HOME = os.getenv("KOAD_HOME", os.path.expanduser("~/.koad-os"))
REDIS_SOCKET = os.path.join(KOAD_HOME, "koad.sock")
GRPC_PORT = 50051
WEB_PORT = 3000

console = Console()

def check_uds():
    try:
        r = redis.Redis(unix_socket_path=REDIS_SOCKET, decode_responses=True)
        r.ping()
        return "PASS", "[green]Reachable[/green]"
    except Exception as e:
        return "FAIL", f"[red]Error: {e}[/red]"

def check_tcp(port):
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=2):
            return "PASS", "[green]Bound (127.0.0.1)[/green]"
    except:
        try:
            # Check 0.0.0.0
            with socket.create_connection(("0.0.0.0", port), timeout=2):
                return "PASS", "[green]Bound (0.0.0.0)[/green]"
        except Exception as e:
            return "FAIL", f"[red]Not Bound ({e})[/red]"

def check_service_inventory():
    try:
        r = redis.Redis(unix_socket_path=REDIS_SOCKET, decode_responses=True)
        services = r.hgetall("koad:services")
        if not services:
            return "WARN", "[yellow]Inventory Empty[/yellow]"
        return "PASS", f"[green]{len(services)} Services Indexed[/green]"
    except:
        return "FAIL", "[red]Inventory Unreachable[/red]"

def run_diagnostics():
    console.print(Panel("[bold cyan]KoadOS Spine Resilience Diagnostic[/bold cyan]\n[dim]Verifying Backbone Stability & Reachability[/dim]", border_style="blue"))
    
    table = Table(title="Spine Reachability Matrix", show_header=True, header_style="bold magenta")
    table.add_column("Service", style="cyan")
    table.add_column("Transport", style="dim")
    table.add_column("Status", justify="center")
    table.add_column("Details")

    # 1. Redis UDS
    status, details = check_uds()
    table.add_row("Redis Kernel", "Unix Socket", status, details)

    # 2. gRPC Gateway (Internal/WSL)
    status, details = check_tcp(GRPC_PORT)
    table.add_row("gRPC Backbone", f"TCP {GRPC_PORT}", status, details)

    # 3. Web Command Deck
    status, details = check_tcp(WEB_PORT)
    table.add_row("Web Deck", f"TCP {WEB_PORT}", status, details)

    # 4. Service Inventory
    status, details = check_service_inventory()
    table.add_row("Service Inventory", "Redis Hash", status, details)

    console.print(table)
    
    # Check overall "Condition Green"
    # Note: rich Table rows are not directly subscriptable like this
    # We'll use a simple list of statuses to track health
    results = [check_uds()[0], check_tcp(GRPC_PORT)[0], check_tcp(WEB_PORT)[0], check_service_inventory()[0]]
    
    if "FAIL" in results:
        console.print(Panel("[bold red]SYSTEM ALERT: SPINE DEGRADED[/bold red]\nBackbone requires Admin attention.", border_style="red"))
        sys.exit(1)
    else:
        console.print(Panel("[bold green]CONDITION GREEN: SPINE STABLE[/bold green]\nBackbone is healthy and reachable.", border_style="green"))

if __name__ == "__main__":
    run_diagnostics()
