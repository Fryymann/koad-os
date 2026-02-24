#!/usr/bin/env python3
import sys
import subprocess
import json

def search_koad(query):
    """
    Queries the KoadOS SQLite database and formats output for Gemini.
    """
    print(f"--- KoadOS Memory Search: '{query}' ---")
    
    try:
        # Run koad query
        result = subprocess.run(
            ["/home/ideans/.koad-os/bin/koad", "query", query],
            capture_output=True,
            text=True,
            check=True
        )
        
        output = result.stdout.strip()
        if not output:
            print(f"No local matches found for '{query}'.")
            return None
            
        print(f"Found matches:\n{output}")
        return output

    except subprocess.CalledProcessError as e:
        print(f"Error querying KoadOS: {e}", file=sys.stderr)
        return None

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: koad skill run gemini/search.py -- "<query_term>"")
        sys.exit(1)
    
    search_term = sys.argv[1]
    search_koad(search_term)
