# KoadOS OSS Implementation Plan

## Vision
Transform KoadOS from a personal toolkit into a cloneable, agent-installed AI persona framework. A user clones the repository, opens their preferred coding agent (Gemini, Claude, etc.), and instructs the agent to "install KoadOS." The agent acts as the installer, interviewing the user to configure the persona, compiling the core, and setting up the operational environment.

## Phase 1: Fork-State Architecture (Continuous Partnership)
KoadOS is designed to live in `~/.koad-os/` as a Git repository.
- **Upstream (OSS)**: The public source of truth.
- **Origin (Personal Fork)**: The user's private extension of KoadOS.
- **Operational Files**: `koad.json`, `koad.db`, and `SESSION_LOG.md` are kept in the root but git-ignored. This ensures that the AI partner's "Memory" and "Identity" stay with the user, even as the "Code" is updated via git.

## Phase 2: The Agent-Driven Installer
The installation is handled by an AI Agent reading `AGENT_INSTALL.md`.
1. **The Interview**: Agent asks the user for their name and the persona's name.
2. **The Setup**: Agent runs `./koad-setup --partner "Ian" --persona "Koad"`.
3. **The Compilation**: The script compiles the Rust core and moves the binary to `~/.koad-os/bin`.
2. **`install.sh` / `install.py`**:
   - A script that accepts command-line arguments (provided by the agent).
   - **Actions**:
     - Compiles the Rust core (`cargo build --release`).
     - Creates the operational directory structure (`~/.koad-os/bin`, `~/.koad-os/skills`, etc.).
     - Copies the compiled `koad` binary, global skills, and drivers to the operational directory.
     - Generates the initial `koad.json` based on the agent's arguments.
     - Initializes an empty `koad.db`.
     - Appends `export PATH="$HOME/.koad-os/bin:$PATH"` to the user's shell profile.

## Phase 3: Core Modifications
Update the Rust binary to cleanly handle the separation of source and state.

1. **Path Resolution**: Ensure `KoadConfig::get_home()` reliably points to the operational directory (e.g., `~/.koad-os/`), regardless of where the `koad` binary is executed from.
2. **Initialization Command (`koad init`)**: Modify this command to handle migrations or re-initialization of the database and configuration without overwriting existing history unless explicitly forced.

## Execution Strategy
1. **Audit**: Review the current `~/.koad-os/` repository to ensure no personal data is currently tracked by Git.
2. **Refactor Paths**: Update `main.rs` if necessary to ensure it expects skills and drivers to live in the operational directory.
3. **Draft Installer**: Create `AGENT_INSTALL.md` and the `install.sh` script.
4. **Test Install**: Clone the repo to a temporary directory and have an agent attempt the installation process to verify the UX.
