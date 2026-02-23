# KoadOS v2.0 (Lean Edition)

The Agnostic AI Coding Agent Framework.

## Setup Instructions

1. **Clone & Personalize**:
   ```bash
   git clone <your-repo-url> ~/.koad-os
   cd ~/.koad-os
   ```

2. **Environment Variables**:
   Add these to your `.bashrc` or `.zshrc` to personalize the persona without editing the code:
   ```bash
   export KOAD_NAME="Your Name"
   export KOAD_ROLE="Your Title"
   export KOAD_BIO="Your background and mission."
   export KOAD_HOME="$HOME/.koad-os"
   ```

3. **Build the CLI**:
   ```bash
   cd core/rust
   cargo build --release
   ```

4. **Initialize Memory**:
   ```bash
   # Create your local koad.json (ignored by git)
   ./target/release/koad init
   ```

5. **Link the Binary**:
   ```bash
   sudo ln -s ~/.koad-os/core/rust/target/release/koad /usr/local/bin/koad
   ```

## Core Mandate
- **Boot**: `koad boot --project` (Ingests persona + recent memory + project snapshot)
- **Remember**: `koad remember fact/learning "<text>"` (Updates global memory)
- **Harvest**: `koad harvest <path>` (PM only: Pulls learnings from developer docs)
- **Skill**: `koad skill run <path>` (Dispatches automation)

## Principles
- **Simplicity first**: Minimal abstractions, maximum speed.
- **Native tech**: Prioritize Rust, Node.js, and Python.
- **Sanctuary Rule**: Developer agents are restricted to project files and documentation.
