import os
import requests
import json
import sys

def get_page_children(page_id):
    token = os.getenv('NOTION_PAT')
    headers = {
        "Authorization": f"Bearer {token}",
        "Notion-Version": "2022-06-28"
    }
    url = f"https://api.notion.com/v1/blocks/{page_id}/children"
    response = requests.get(url, headers=headers)
    if response.status_code == 200:
        return response.json().get("results", [])
    return []

def archive_block(block_id):
    token = os.getenv('NOTION_PAT')
    headers = {
        "Authorization": f"Bearer {token}",
        "Notion-Version": "2022-06-28"
    }
    url = f"https://api.notion.com/v1/blocks/{block_id}"
    requests.delete(url, headers=headers)

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
    return response.status_code

if __name__ == "__main__":
    page_id = "30cfe8ec-ae8f-808b-8eff-fa75e1cb0572"
    
    # 1. Archive existing content
    print("Archiving old content...")
    children = get_page_children(page_id)
    for child in children:
        archive_block(child["id"])
    
    # 2. Define new content based on koad.json v2.3
    blocks = [
        {
            "object": "block",
            "type": "heading_1",
            "heading_1": {
                "rich_text": [{"type": "text", "text": {"content": "KoadOS Core Contract (v2.3)"}}]
            }
        },
        {
            "object": "block",
            "type": "callout",
            "callout": {
                "rich_text": [{"type": "text", "text": {"content": "Identity: Principal Systems & Operations Engineer\\nAgnostic AI persona optimized for simplicity, reliability, and script-driven automation."}}],
                "icon": {"type": "emoji", "emoji": "\ud83e\uddd1\u200d\ud83d\ude80"},
                "color": "blue_background"
            }
        },
        {
            "object": "block",
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": "The Prime Directives"}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Simplicity over complexity."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Plan before build."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Native tech focus."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Programmatic-first communication."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Sanctuary Rule: Developer agents only touch project files & docs."}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "Rust Stack: Prioritize cargo-llvm-cov, ratatui, cargo-doc, and cargo-deadlinks."}}]
            }
        },
        {
            "object": "block",
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": "Operational Infrastructure"}}]
            }
        },
        {
            "object": "block",
            "type": "code",
            "code": {
                "language": "yaml",
                "rich_text": [{"type": "text", "text": {"content": "binary: ~/.koad-os/bin/koad\\nmemory: ~/.koad-os/koad.db\\nconfig: ~/.koad-os/koad.json\\nstream: 310fe8ec-ae8f-80ba-9cbb-f31731d396d4"}}]
            }
        },
        {
            "object": "block",
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": "Communication Protocols"}}]
            }
        },
        {
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [
                    {"type": "text", "text": {"content": "All asynchronous delegation between Koad (CLI) and Noti (Notion AI) occurs in the "}},
                    {"type": "mention", "mention": {"type": "database", "database": {"id": "310fe8ec-ae8f-80ba-9cbb-f31731d396d4"}}}
                ]
            }
        },
        {
            "object": "block",
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": "Sync Index"}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "koad: 30cfe8ec-ae8f-808b-8eff-fa75e1cb0572"}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "stream: 310fe8ec-ae8f-80ba-9cbb-f31731d396d4"}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "noti: 295fe8ec-ae8f-805d-a0c8-e44bf3bbef0b"}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "projects: 2b5cf778-395b-4ac8-8775-b6b80c3cdf2f"}}]
            }
        },
        {
            "object": "block",
            "type": "bulleted_list_item",
            "bulleted_list_item": {
                "rich_text": [{"type": "text", "text": {"content": "memories: ae366b72-8cd2-4da2-a242-a1f2d6cae343"}}]
            }
        }
    ]
    
    # Note: Adding table rows requires a separate call to children of the table block. 
    # For now, I will append the main blocks.
    
    print("Appending new content...")
    status = append_blocks(page_id, blocks)
    if status == 200:
        print("Koad page refreshed successfully.")
    else:
        print(f"Failed to append blocks: {status}")
