import pytest
import os

def test_project_registration_and_list(koad_env):
    """Verify that projects can be registered and listed."""
    # 1. Register a project
    reg_res = koad_env.run_koad(["project", "register", "test-project", str(koad_env.root_dir)])
    assert reg_res.returncode == 0
    assert "Project 'test-project' registered" in reg_res.stdout

    # 2. List projects
    list_res = koad_env.run_koad(["project", "list"])
    assert list_res.returncode == 0
    assert "test-project" in list_res.stdout
    assert str(koad_env.root_dir) in list_res.stdout

def test_project_sync(koad_env):
    """Verify that project sync updates branch and health."""
    # 1. Register current koad_home as a project
    koad_env.run_koad(["project", "register", "koad-home", str(koad_env.koad_home)])
    
    # 2. Sync it
    # We are inside the koad_home (via KOAD_HOME env), but let's use ID 1 for simplicity in a fresh env
    sync_res = koad_env.run_koad(["project", "sync", "1"])
    assert sync_res.returncode == 0
    assert "Project #1 status updated" in sync_res.stdout

    # 3. Check list for updates
    list_res = koad_env.run_koad(["project", "list"])
    assert "green" in list_res.stdout # Should be green because koad.json exists

def test_project_retire(koad_env):
    """Verify that projects can be retired from the map."""
    koad_env.run_koad(["project", "register", "to-retire", str(koad_env.root_dir)])
    
    # Retire it
    ret_res = koad_env.run_koad(["project", "retire", "1"])
    assert ret_res.returncode == 0
    
    # Should no longer be in list
    list_res = koad_env.run_koad(["project", "list"])
    assert "to-retire" not in list_res.stdout

def test_project_info(koad_env):
    """Verify detailed project info retrieval."""
    koad_env.run_koad(["project", "register", "info-test", str(koad_env.root_dir)])
    
    info_res = koad_env.run_koad(["project", "info", "1"])
    assert info_res.returncode == 0
    assert "Project Info: info-test" in info_res.stdout
    assert str(koad_env.root_dir) in info_res.stdout
