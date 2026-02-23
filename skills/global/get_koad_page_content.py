import os
import requests
import json
import sys

def get_page_content(page_id):
    token = os.getenv('NOTION_PAT')
    headers = {
        "Authorization": f"Bearer {token}",
        "Notion-Version": "2022-06-28"
    }
    response = requests.get(f"https://api.notion.com/v1/blocks/{page_id}/children", headers=headers)
    if response.status_code == 200:
        return response.json()
    else:
        print(f"Error: {response.status_code}")
        print(response.text)
        return None

if __name__ == "__main__":
    page_id = "30cfe8ec-ae8f-808b-8eff-fa75e1cb0572"
    content = get_page_content(page_id)
    if content:
        print(json.dumps(content, indent=2))
