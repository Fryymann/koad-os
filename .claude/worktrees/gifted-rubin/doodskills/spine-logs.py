#!/home/ideans/.koad-os/venv/bin/python3
import redis
import json
import time
import sys
import os
from datetime import datetime
from rich.console import Console
from rich.table import Table
from rich.live import Live
from rich.panel import Panel
from rich.text import Text

# Configuration
KOAD_HOME = os.getenv("KOAD_HOME", os.path.expanduser("~/.koad-os"))
REDIS_SOCKET = os.path.join(KOAD_HOME, "koad.sock")
STREAM_KEY = "koad:events:stream"

console = Console()

def format_timestamp(ts):
    try:
        dt = datetime.fromtimestamp(float(ts))
        return dt.strftime("%H:%M:%S")
    except:
        return ts

def get_severity_style(severity):
    styles = {
        "DEBUG": "dim cyan",
        "INFO": "green",
        "WARN": "yellow",
        "ERROR": "bold red",
        "CRITICAL": "bold white on red"
    }
    return styles.get(severity, "white")

def tail_spine():
    try:
        # Connect via UDS
        r = redis.Redis(unix_socket_path=REDIS_SOCKET, decode_responses=True)
        
        console.print(Panel("[bold green]KoadOS Spine Diagnostic Monitor[/bold green]\n[dim]Tailing koad:events:stream via UDS[/dim]", border_style="blue"))
        
        # Start from the end of the stream
        last_id = "$"
        
        while True:
            # Read new events (block for 1 second)
            events = r.xread({STREAM_KEY: last_id}, count=10, block=1000)
            
            if not events:
                continue
                
            for _, message_list in events:
                for msg_id, payload in message_list:
                    last_id = msg_id
                    
                    source = payload.get("source", "unknown")
                    severity = payload.get("severity", "INFO")
                    message = payload.get("message", "---")
                    metadata_raw = payload.get("metadata", "{}")
                    ts = payload.get("timestamp", time.time())
                    
                    # Format Output
                    time_str = format_timestamp(ts)
                    style = get_severity_style(severity)
                    
                    msg_text = Text()
                    msg_text.append(f"[{time_str}] ", style="dim")
                    msg_text.append(f"{severity:<8} ", style=style)
                    msg_text.append(f"{source:<20} ", style="cyan")
                    msg_text.append(f"| {message}", style="white")
                    
                    console.print(msg_text)
                    
                    # If it's a heartbeat, show a mini-summary
                    if message == "SYSTEM_HEARTBEAT":
                        try:
                            meta = json.loads(metadata_raw)
                            console.print(f"  [dim]CPU: {meta.get('cpu_usage', 0):.1f}% | MEM: {meta.get('memory_usage', 0)}MB | UPTIME: {meta.get('uptime', 0)}s[/dim]")
                        except:
                            pass

    except KeyboardInterrupt:
        console.print("\n[bold yellow]Monitoring suspended by Admin.[/bold yellow]")
    except Exception as e:
        console.print(f"\n[bold red]ERROR:[/bold red] Failed to connect to Spine: {e}")
        sys.exit(1)

if __name__ == "__main__":
    tail_spine()
