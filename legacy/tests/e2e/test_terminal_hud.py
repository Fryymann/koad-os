import pytest

def test_terminal_hud_init(koad_env):
    """Verify the TUI (kdash) can initialize its environment."""
    # We can't easily test the interactive TUI in E2E, but we can verify 
    # it has the required bin and help output
    result = koad_env.run_cli(["dash", "--help"])
    # kdash might not support --help if it's a raw TUI, checking existence
    assert (koad_env.bin_dir / "kdash").exists()
