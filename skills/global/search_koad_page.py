import os
import requests
import json

def search_notion(query):
    token = os.getenv('NOTION_PAT')
    if not token:
        print("NOTION_PAT not found")
        return None
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "Notion-Version": "2022-06-28"
    }
    
    payload = {
        "query": query,
        "filter": {
            "value": "page",
            "property": "object"
        }
    }
    
    response = requests.post("https://api.notion.com/v1/search", headers=headers, json=payload)
    if response.status_code == 200:
        return response.json()
    else:
        print(f"Error: {response.status_code}")
        print(response.text)
        return None

if __name__ == "__main__":
    results = search_notion("Koad")
    if results:
        print(json.dumps(results, indent=2))
