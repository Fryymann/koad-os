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
    # Koad page
    page_id = "30cfe8ec-ae8f-808b-8eff-fa75e1cb0572"
    
    blocks = [
        {
            "object": "block",
            "type": "heading_3",
            "heading_3": {
                "rich_text": [{"type": "text", "text": {"content": "DELEGATION STREAM (CLI \u2194 Noti)"}}]
            }
        },
        {
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [
                    {
                        "type": "text", 
                        "text": {"content": "Koad uses the "}
                    },
                    {
                        "type": "mention",
                        "mention": {
                            "type": "database",
                            "database": {"id": "310fe8ec-ae8f-80ba-9cbb-f31731d396d4"}
                        }
                    },
                    {
                        "type": "text",
                        "text": {"content": " to broadcast operational state and delegate admin tasks to Noti."}
                    }
                ]
            }
        },
        {
            "object": "block",
            "type": "code",
            "code": {
                "language": "bash",
                "rich_text": [
                    {"type": "text", "text": {"content": "# Post a log\\nkoad stream post \"Sync Complete\" \"All local facts synced to Airtable.\"\\n\\n# Ask Noti a question\\nkoad stream post \"Invoice Audit\" \"Noti, please check for unpaid MemberPlanet invoices from last week.\" --type Question"}}
                ]
            }
        }
    ]
    
    append_blocks(page_id, blocks)
