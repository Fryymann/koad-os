import os
import requests
import json

def search_notion(query):
    token = os.getenv('NOTION_PAT')
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "Notion-Version": "2022-06-28"
    }
    payload = {"query": query}
    response = requests.post("https://api.notion.com/v1/search", headers=headers, json=payload)
    if response.status_code == 200:
        return response.json()
    else:
        return None

if __name__ == "__main__":
    results = search_notion("Noti")
    if results:
        print(json.dumps(results, indent=2))
