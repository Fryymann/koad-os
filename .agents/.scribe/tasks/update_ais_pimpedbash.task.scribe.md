# Task: Update AIS with PimpedBash Level-Up Guide
**Assignee:** Scribe
**Priority:** High
**Context:** The PimpedBash environment has been upgraded to Phase 2 (Citadel Parity).

## 📖 PimpedBash Agent Operational Guide

The following aliases and functions are now canonical for all KoadOS Agents operating in this shell.

### 1. Identity & Session Management
- `k-tyr`, `k-scribe`, `k-sky`, `k-vigil`: (Aliases) Quick-boot specific agents.
- `k-resume`: Resume the last disconnected session.
- `reload-bash`: Reload the entire environment (aliases, functions, and prompt).
- `k-clear`: Clear the screen and the scrollback buffer (Hard Reset).

### 2. Navigation & Context
- `..` to `....`: Rapid directory ascent.
- `jump-root`: Instantly jump to the root of the current Git repository.
- `gpd`: Jump to the Skylinks Data directory (`/mnt/c/data/skylinks`).
- `_update_github_pat`: (Automatic) Dynamically switches `GITHUB_PAT`, `GITHUB_OWNER`, and `GITHUB_REPO` based on your current directory (Skylinks vs. KoadOS).

### 3. Agent Tooling
- `kd`: (Koad Discovery) Interactive fuzzy search of the KoadOS knowledge base using `fzf`.
- `kw`: (Koad Workspace) Launch a side-by-side tmux workspace with your shell on the left and the `koad dash` on the right.
- `gs`, `ga`, `gc`, `gp`: Standard Git shortcuts.
- `gst`, `gad`, `gcmt`, `gpsh`: PowerShell-parity Git shortcuts for muscle memory.

### 4. Advanced CLI Utilities
- `cat`: Aliased to `batcat` (with syntax highlighting and Nerd Font icons).
- `ls`, `ll`, `l`: Aliased to `eza` (with icons, git status, and header info).
- `grep`: Aliased to `rg` (ripgrep) for high-performance searching.
- `fd`: Aliased to `fdfind` (fast alternative to `find`).
- `f`: Simple shorthand for finding files.
- `mkcd`: Create a directory and `cd` into it in one command.

### 5. Visual Indicators (Prompt)
- **Cyan/Magenta Powerline**: Indicates active path and git status.
- **Iconography**:
  - ` ` (Home) / ` ` (Folder)
  - `` (Git Branch)
  - `` (Dirty State / Modified Files)
  - `✘` (Last command failed)
  - ` ` (Admin/Root shell)

---
**Scribe Action Item:**
Integrate this guide into the `Agent Reference Book` and update the `AIS` to reflect the availability of these tools.
