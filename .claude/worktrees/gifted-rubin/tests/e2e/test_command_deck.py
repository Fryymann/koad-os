import pytest

def test_command_deck_connectivity(gateway):
    """Verify the Web Deck gateway accepts connections."""
    # The gateway fixture starts the gateway on a free port
    import socket
    with socket.create_connection(("127.0.0.1", gateway.gateway_port), timeout=1.0) as s:
        assert s is not None
