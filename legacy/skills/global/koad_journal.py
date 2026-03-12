#!/usr/bin/env python3
import sys
import subprocess
import argparse

def main():
    parser = argparse.ArgumentParser(description="Koad's Persona Journal")
    parser.add_argument('action', choices=['write', 'read'])
    parser.add_argument('--entry', help="The content of the journal entry")
    args = parser.parse_args()

    if args.action == 'write':
        if not args.entry:
            print("Error: Entry content required.")
            sys.exit(1)
        
        # We record this as a 'learning' with a specific tag
        subprocess.run([
            "koad", "remember", "learning", 
            args.entry, 
            "--tags", "persona-journal,reflection"
        ], check=True)
        print("Perspective recorded. Koad's internal state updated.")

    elif args.action == 'read':
        # Retrieve the last 5 journal entries
        subprocess.run([
            "koad", "query", "persona-journal"
        ], check=True)

if __name__ == "__main__":
    main()
