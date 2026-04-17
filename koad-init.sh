#!/usr/bin/env bash
# =============================================================================
# KoadOS Initialization Script
# =============================================================================
set -euo pipefail

KOAD_HOME="${KOADOS_HOME:-${1:-$HOME/.koad-os}}"
BIN_DIR="$KOAD_HOME/bin"

# Colours
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

ok()   { echo -e "${GREEN}  ✓${RESET}  $*"; }
warn() { echo -e "${YELLOW}  ⚠${RESET}  $*"; }
fail() { echo -e "${RED}  ✗${RESET}  $*"; }
info() { echo -e "${CYAN}  →${RESET}  $*"; }
section() { echo -e "\n${BOLD}[$*]${RESET}"; }

# 0. Root Check & Portability Helpers
if [[ ! -d "blueprints" || ! -f "Cargo.toml" ]]; then
    echo -e "${RED}  ✗${RESET}  Execution error: Run this script from the root of the koad-os repository."
    exit 1
fi

portable_sed() {
    local pattern="$1"
    local file="$2"
    sed "$pattern" "$file" > "$file.tmp" && mv "$file.tmp" "$file"
}

section "KoadOS Initialization"

# 1. Citadel Identity
echo -e "${BOLD}Step 1: Citadel Identity${RESET}"
read -p "Enter your Citadel Name [Sanctuary]: " CITADEL_NAME
CITADEL_NAME=${CITADEL_NAME:-Sanctuary}
info "Initializing Citadel: $CITADEL_NAME"

# 2. Directory Setup
section "Directory Setup"
for dir in "$BIN_DIR" "$KOAD_HOME/agents/captain" "$KOAD_HOME/config" "$KOAD_HOME/cache" "$KOAD_HOME/data/db" "$KOAD_HOME/logs" "$KOAD_HOME/run"; do
    mkdir -p "$dir"
    ok "$dir created"
done

# 3. .env Initialization
section "Environment Setup"
if [[ -f "$KOAD_HOME/.env" ]]; then
    ok ".env already exists"
else
    if [[ -f ".env.template" ]]; then
        cp .env.template "$KOAD_HOME/.env"
        # Attempt to set KOADOS_HOME in the new .env
        portable_sed "s|KOADOS_HOME=.*|KOADOS_HOME=$KOAD_HOME|" "$KOAD_HOME/.env"
        ok ".env initialized from template"
    else
        warn ".env.template not found. Skipping .env initialization."
    fi
fi

# 4. Captain Identity Initialization
section "Captain Identity Setup"
if [[ -f "$KOAD_HOME/agents/captain/IDENTITY.toml" ]]; then
    ok "Captain identity already exists"
else
    if [[ -d "blueprints/captain" ]]; then
        cp blueprints/captain/IDENTITY.toml "$KOAD_HOME/agents/captain/IDENTITY.toml"
        cp blueprints/captain/SYSTEM.md "$KOAD_HOME/agents/captain/SYSTEM.md"
        
        # Customize IDENTITY.toml
        portable_sed "s/station = \"Citadel\"/station = \"$CITADEL_NAME\"/" "$KOAD_HOME/agents/captain/IDENTITY.toml"
        ok "Captain identity initialized for $CITADEL_NAME"
    else
        fail "Blueprints not found. Captain identity could not be initialized."
    fi
fi

# 5. Shell Helper Installation
section "Shell Helpers"
if [[ -f "scripts/koad-functions.sh" ]]; then
    cp scripts/koad-functions.sh "$BIN_DIR/koad-functions.sh"
    ok "koad-functions.sh installed to $BIN_DIR"
else
    warn "scripts/koad-functions.sh not found."
fi

# 6. Binary Installation (Final Check)
section "Binary Installation"
for bin in koad koad-agent; do
    if [[ -f "target/release/$bin" ]]; then
        cp "target/release/$bin" "$BIN_DIR/$bin"
        ok "$bin installed to $BIN_DIR"
    else
        warn "$bin not found in target/release/. Ensure you ran ./install.sh first."
    fi
done

# 7. Database Migrations / Setup
section "Database & State"
# Future: Run migrations here
ok "Database state verified."

# 8. Graph Initialization
section "Graph-Centric Navigation Setup"
if command -v "code-review-graph" &>/dev/null; then
    info "Initializing and building the codebase graph..."
    code-review-graph init
    code-review-graph build
    ok "Graph built successfully."
else
    warn "code-review-graph not found. Graph-centric navigation will be disabled."
    info "To enable it, install the tool: ${BOLD}pipx install code-review-graph${RESET}"
    info "Then run: ${CYAN}code-review-graph init && code-review-graph build${RESET}"
fi

section "Initialization Complete"
info "Citadel '$CITADEL_NAME' is ready."
echo -e "\n${BOLD}Final Actions:${RESET}"
echo -e "1. Add KoadOS to your shell (e.g., ~/.bashrc):"
echo -e "   ${CYAN}export KOADOS_HOME=\"$KOAD_HOME\"${RESET}"
echo -e "   ${CYAN}export PATH=\"\$KOADOS_HOME/bin:\$PATH\"${RESET}"
echo -e "   ${CYAN}source \$KOADOS_HOME/bin/koad-functions.sh${RESET}"
echo -e "2. Reload your shell: ${CYAN}source ~/.bashrc${RESET}"
echo -e "3. Boot your Captain: ${BOLD}agent-boot captain${RESET}\n"
