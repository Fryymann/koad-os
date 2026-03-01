import grpc
from concurrent import futures
import time
import os
from .proto import skill_pb2_grpc, skill_pb2

class Skill:
    """The KoadOS Skill SDK. Allows Python scripts to register as gRPC services."""

    def __init__(self, skill_id: str, socket_path: str = "/tmp/koad-skill.sock"):
        self.skill_id = skill_id
        self.socket_path = socket_path
        self.server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
        
    def add_service(self, servicer):
        """Adds the skill implementation to the gRPC server."""
        skill_pb2_grpc.add_SkillServiceServicer_to_server(servicer, self.server)

    def start(self):
        """Starts the skill's gRPC listener."""
        # Ensure stale sockets are removed
        if os.path.exists(self.socket_path):
            os.remove(self.socket_path)
            
        self.server.add_insecure_port(f"unix://{self.socket_path}")
        self.server.start()
        print(f"Skill [{self.skill_id}] listening on {self.socket_path}")
        
    def wait_for_termination(self):
        """Keeps the skill process alive."""
        self.server.wait_for_termination()
