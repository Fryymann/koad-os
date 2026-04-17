#!/usr/bin/env bash
# =============================================================================
# KoadOS Unified Stable Installer
# =============================================================================
set -euo pipefail

# Colours
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

ok()   { echo -e "${GREEN}  ✓${RESET}  $*"; }
warn() { echo -e "${YELLOW}  ⚠${RESET}  $*"; }
fail() { echo -e "${RED}  ✗${RESET}  $*"; }
info() { echo -e "${CYAN}  →${RESET}  $*"; }
section() { echo -e "\n${BOLD}[$*]${RESET}"; }

echo -e "${BOLD}"
echo "  ██╗  ██╗ ██████╗  █████╗ ██████╗      ██████╗ ███████╗"
echo "  ██║ ██╔╝██╔═══██╗██╔══██╗██╔══██╗    ██╔═══██╗██╔════╝"
echo "  █████╔╝ ██║   ██║███████║██║  ██║    ██║   ██║███████╗"
echo "  ██╔═██╗ ██║   ██║██╔══██║██║  ██║    ██║   ██║╚════██║"
echo "  ██║  ██╗╚██████╔╝██║  ██║██████╔╝    ╚██████╔╝███████║"
echo "  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝      ╚═════╝ ╚══════╝"
echo -e "${RESET}"
echo "  Unified Stable Installer  ·  v3.2.0"
echo

# 1. Prerequisite Detection
section "Prerequisite Detection"
ERRORS=0

check_cmd() {
    if command -v "$1" &>/dev/null; then
        ok "$1 found ($(command -v "$1"))"
    else
        fail "$1 not found. Please install it to continue."
        ERRORS=$((ERRORS + 1))
    fi
}

check_cmd "rustc"
check_cmd "cargo"
check_cmd "docker"
check_cmd "protoc"

# Check for docker-compose or docker compose
if command -v "docker-compose" &>/dev/null; then
    ok "docker-compose found"
elif docker compose version &>/dev/null; then
    ok "docker compose plugin found"
else
    fail "docker-compose or docker compose plugin not found."
    ERRORS=$((ERRORS + 1))
fi

check_cmd "python3"
check_cmd "pipx"

# 1.1 specialized check for code-review-graph
if command -v "code-review-graph" &>/dev/null; then
    ok "code-review-graph found ($(command -v "code-review-graph"))"
else
    fail "code-review-graph not found."
    info "Please install it via pipx: ${BOLD}pipx install code-review-graph${RESET}"
    ERRORS=$((ERRORS + 1))
fi

if [[ $ERRORS -gt 0 ]]; then
    echo
    fail "$ERRORS prerequisite(s) missing. Fix the above and re-run $0."
    exit 1
fi

# 2. Infrastructure Boot
section "Infrastructure Boot (Docker)"
info "Starting CASS, Redis, and Qdrant..."
if command -v "docker-compose" &>/dev/null; then
    docker-compose up -d --build
else
    docker compose up -d --build
fi
ok "Infrastructure is running in the background."

# 3. Host Binary Compilation
section "Host Binary Compilation (Rust)"
info "Building 'koad' and 'koad-agent' in release mode..."
cargo build --release --bin koad --bin koad-agent
ok "Binaries compiled successfully."

# 4. Next Steps
section "Installation Phase 1 Complete"
info "Infrastructure is online and binaries are built."
echo -e "\n${BOLD}Next Step:${RESET}"
echo -e "Run the initialization script to set up your Citadel identity:"
echo -e "  ${CYAN}./koad-init.sh${RESET}\n"
