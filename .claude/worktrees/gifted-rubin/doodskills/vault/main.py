from koad.skill import Skill
from koad.proto import skill_pb2, skill_pb2_grpc
import time
import os

class VaultServicer(skill_pb2_grpc.SkillServiceServicer):
    def Run(self, request, context):
        print(f"Vault Skill: Running tool {request.skill_id}")
        
        if request.skill_id == "snapshot":
            # Placeholder: Migration of legacy vault logic
            output = "Vault snapshot created successfully (v3 SDK)."
        elif request.skill_id == "list":
            output = "Vault: [snapshot_20260228, snapshot_20260227]"
        else:
            output = f"Vault: Tool {request.skill_id} not implemented."

        yield skill_pb2.RunUpdate(
            skill_id=request.skill_id,
            output=output,
            progress=1.0,
            finished=True
        )

def main():
    sdk = Skill("vault", socket_path="/tmp/koad-vault.sock")
    sdk.add_service(VaultServicer())
    sdk.start()
    sdk.wait_for_termination()

if __name__ == "__main__":
    main()
