#!/usr/bin/env python3
import sqlite3
import os
import sys
import argparse
from datetime import datetime

# Path to the KoadOS database
DB_PATH = os.path.expanduser("~/.koad-os/koad.db")

def get_conn():
    return sqlite3.connect(DB_PATH)

def pin_workflow(title, description=None, project=None, priority=0):
    now = datetime.now().isoformat()
    conn = get_conn()
    cursor = conn.cursor()
    cursor.execute(
        "INSERT INTO workflows (title, description, status, priority, project, last_update) VALUES (?, ?, ?, ?, ?, ?)",
        (title, description, 'Pinned', priority, project, now)
    )
    conn.commit()
    conn.close()
    print(f"Workflow '{title}' pinned to KoadOS.")

def list_workflows(status=None):
    conn = get_conn()
    cursor = conn.cursor()
    if status:
        cursor.execute("SELECT id, title, status, project FROM workflows WHERE status = ? ORDER BY priority DESC", (status,))
    else:
        cursor.execute("SELECT id, title, status, project FROM workflows ORDER BY status, priority DESC")
    
    rows = cursor.fetchall()
    conn.close()
    
    if not rows:
        print("No workflows found.")
    else:
        print(f"{'ID':<4} | {'Title':<40} | {'Status':<10} | {'Project'}")
        print("-" * 75)
        for row in rows:
            print(f"{row[0]:<4} | {row[1]:<40} | {row[2]:<10} | {row[3] or 'N/A'}")

def activate_workflow(wf_id):
    conn = get_conn()
    cursor = conn.cursor()
    cursor.execute("SELECT title, description FROM workflows WHERE id = ?", (wf_id,))
    row = cursor.fetchone()
    if row:
        title, desc = row
        # Update current active spec (id=1) for the Rust binary
        now = datetime.now().isoformat()
        cursor.execute(
            "INSERT INTO active_spec (id, title, description, status, last_update) VALUES (1, ?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET title=?1, description=?2, status=?3, last_update=?4",
            (title, desc, 'Active', now)
        )
        # Update workflow status to Active
        cursor.execute("UPDATE workflows SET status = 'Active', last_update = ? WHERE id = ?", (now, wf_id))
        conn.commit()
        print(f"Workflow {wf_id} ('{title}') is now ACTIVE.")
    else:
        print(f"Workflow ID {wf_id} not found.")
    conn.close()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="KoadOS Workflow Manager")
    subparsers = parser.add_subparsers(dest="command")

    # Pin
    pin_parser = subparsers.add_parser("pin", help="Pin a new workflow")
    pin_parser.add_argument("title", help="Title of the workflow")
    pin_parser.add_argument("--description", help="Detailed description")
    pin_parser.add_argument("--project", help="Associated project name")
    pin_parser.add_argument("--priority", type=int, default=0, help="Priority level (default 0)")

    # List
    list_parser = subparsers.add_parser("list", help="List workflows")
    list_parser.add_argument("--status", help="Filter by status (e.g., Pinned, Pending, Active)")

    # Activate
    activate_parser = subparsers.add_parser("activate", help="Activate a pinned workflow")
    activate_parser.add_argument("id", type=int, help="ID of the workflow to activate")

    args = parser.parse_args()

    if args.command == "pin":
        pin_workflow(args.title, args.description, args.project, args.priority)
    elif args.command == "list":
        list_workflows(args.status)
    elif args.command == "activate":
        activate_workflow(args.id)
    else:
        parser.print_help()
