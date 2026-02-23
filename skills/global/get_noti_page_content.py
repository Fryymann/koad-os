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
        return None

if __name__ == "__main__":
    page_id = "295fe8ec-ae8f-805d-a0c8-e44bf3bbef0b"
    content = get_page_content(page_id)
    if content:
        print(json.dumps(content, indent=2))
