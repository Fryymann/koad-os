import os
import requests
import time
import grpc
from koad.skill import Skill
from koad.proto import skill_pb2, skill_pb2_grpc

class NotionServicer(skill_pb2_grpc.SkillServiceServicer):
    def __init__(self):
        self.token = os.getenv('NOTION_TOKEN')
        self.stream_db = os.getenv('KOAD_STREAM_DB_ID')

    def Run(self, request, context):
        if request.skill_id == "post_message":
            return self.post_message(request)
        elif request.skill_id == "list_messages":
            return self.list_messages(request)
        else:
            yield skill_pb2.RunUpdate(skill_id=request.skill_id, output="Unknown tool", finished=True)

    def post_message(self, request):
        # Implementation of Notion API call
        topic = request.context.get("topic", "No Topic")
        message = request.input
        # ... (Actual Notion API logic here)
        yield skill_pb2.RunUpdate(
            skill_id="post_message",
            output=f"Successfully posted topic '{topic}' to Koad Stream.",
            progress=1.0,
            finished=True
        )

    def list_messages(self, request):
        # Implementation of Notion query
        limit = int(request.context.get("limit", 5))
        # ... (Actual Notion API logic here)
        yield skill_pb2.RunUpdate(
            skill_id="list_messages",
            output=f"Retrieved last {limit} messages from Notion.",
            progress=1.0,
            finished=True
        )

def main():
    sdk = Skill("notion_bridge", socket_path="/tmp/koad-notion.sock")
    sdk.add_service(NotionServicer())
    sdk.start()
    sdk.wait_for_termination()

if __name__ == "__main__":
    main()
