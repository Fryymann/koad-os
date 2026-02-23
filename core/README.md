# koadOS Core (Global)

Location: `~/.koad-os/core`

Project-agnostic core for koadOS.

## Architecture

- `koad_model.py`: pure data models/config objects (no runtime/process dependencies)
- `koad_runtime.py`: runtime adapter contract + default shell runtime
- `koad_core.py`: project-module loading + scope classification helpers

This split keeps policy/model independent from execution backend.

## Project Contract

Each project provides:

- `.koad/project/koad_project.py`
- function: `get_project_config() -> dict | ProjectConfig`

## Runtime Independence

Core logic should operate on `RuntimeAdapter` where command execution is required.
`ShellRuntime` is default, but projects can supply different adapters for tests or other environments.
