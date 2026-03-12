from .proto import kernel_pb2_grpc, kernel_pb2
import grpc

class KernelClient:
    """A client for Skills to talk back to the KoadOS Kernel."""
    
    def __init__(self, kernel_socket: str = "/home/ideans/.koad-os/kspine.sock"):
        self.channel = grpc.insecure_channel(f"unix://{kernel_socket}")
        self.stub = kernel_pb2_grpc.KernelServiceStub(self.channel)

    def execute(self, name: str, args: dict = None, identity: str = "crew"):
        """Sends a command to the Kernel for execution."""
        request = kernel_pb2.CommandRequest(
            name=name,
            args=args or {},
            identity=identity
        )
        return self.stub.Execute(request)
