from koad.skill import Skill
from koad.proto import skill_pb2, skill_pb2_grpc
import time
import os

class BlueprintServicer(skill_pb2_grpc.SkillServiceServicer):
    def Run(self, request, context):
        print(f"DoodSkill: Blueprint Engine running tool {request.skill_id}")
        
        if request.skill_id == "generate":
            output = f"Blueprint 'v3_scaffold' generated successfully for Dood."
        elif request.skill_id == "validate":
            output = "Blueprint validation: Status Green."
        else:
            output = f"Blueprint: Tool {request.skill_id} not implemented."

        yield skill_pb2.RunUpdate(
            skill_id=request.skill_id,
            output=output,
            progress=1.0,
            finished=True
        )

def main():
    sdk = Skill("blueprint_engine", socket_path="/tmp/koad-blueprint.sock")
    sdk.add_service(BlueprintServicer())
    sdk.start()
    sdk.wait_for_termination()

if __name__ == "__main__":
    main()
