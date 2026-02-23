import os
import requests
import json
import sys

def append_blocks(page_id, blocks):
    token = os.getenv('NOTION_PAT')
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "Notion-Version": "2022-06-28"
    }
    url = f"https://api.notion.com/v1/blocks/{page_id}/children"
    payload = {"children": blocks}
    response = requests.patch(url, headers=headers, json=payload)
    if response.status_code == 200:
        print("Successfully appended blocks.")
    else:
        print(f"Error: {response.status_code}")
        print(response.text)

if __name__ == "__main__":
    # Noti Agent OS page
    page_id = "295fe8ec-ae8f-805d-a0c8-e44bf3bbef0b"
    
    blocks = [
        {
            "object": "block",
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": "COMMUNICATION PROTOCOLS (v1)"}}]
            }
        },
        {
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [
                    {
                        "type": "text", 
                        "text": {"content": "Noti and Koad (CLI) communicate via the "}
                    },
                    {
                        "type": "mention",
                        "mention": {
                            "type": "database",
                            "database": {"id": "310fe8ec-ae8f-80ba-9cbb-f31731d396d4"}
                        }
                    }
                ]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Noti check: Whenever active, Noti should check the stream for 'Unread' topics addressed to her or tagged 'Question'."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Acknowledgment: When Noti acts on a stream item, she moves Status to 'Acknowledged' and posts a follow-up if needed."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Pulse Posts: Koad CLI posts operational logs and facts to the stream as 'Log' types for Noti's ingestion."}}]
            }
        }
    ]
    
    append_blocks(page_id, blocks)
