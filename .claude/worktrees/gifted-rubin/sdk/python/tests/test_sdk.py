import grpc
import unittest
import os
from koad.proto import kernel_pb2, kernel_pb2_grpc

class TestSDK(unittest.TestCase):
    def test_proto_import(self):
        """Verifies that the generated proto modules are importable."""
        self.assertIsNotNone(kernel_pb2.CommandRequest)
        self.assertIsNotNone(kernel_pb2_grpc.KernelServiceStub)

if __name__ == "__main__":
    unittest.main()
