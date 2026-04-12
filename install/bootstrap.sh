#!/usr/bin/env bash
# =============================================================================
# KoadOS Bootstrap — First-time setup after `git clone`
# Fulfills Task 1.2: Idempotent and Portable Bootstrap
# =============================================================================
set -euo pipefail

# bootstrap.sh lives in install/ — KOAD_HOME is its parent directory
KOAD_HOME="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$KOAD_HOME/bin"
LOG_DIR="$KOAD_HOME/logs"

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

ok()   { echo -e "${GREEN}  ✓${RESET}  $*"; }
warn() { echo -e "${YELLOW}  ⚠${RESET}  $*"; }
fail() { echo -e "${RED}  ✗${RESET}  $*"; }
info() { echo -e "${CYAN}  →${RESET}  $*"; }
section() { echo -e "\n${BOLD}[$*]${RESET}"; }

# ── Flags ─────────────────────────────────────────────────────────────────────
YES_MODE=false
for arg in "$@"; do
    [[ "$arg" == "--yes" || "$arg" == "-y" ]] && YES_MODE=true
done

echo -e "${BOLD}"
echo "  ██╗  ██╗ ██████╗  █████╗ ██████╗      ██████╗ ███████╗"
echo "  ██║ ██╔╝██╔═══██╗██╔══██╗██╔══██╗    ██╔═══██╗██╔════╝"
echo "  █████╔╝ ██║   ██║███████║██║  ██║    ██║   ██║███████╗"
echo "  ██╔═██╗ ██║   ██║██╔══██║██║  ██║    ██║   ██║╚════██║"
echo "  ██║  ██╗╚██████╔╝██║  ██║██████╔╝    ╚██████╔╝███████║"
echo "  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝      ╚═════╝ ╚══════╝"
echo -e "${RESET}"
echo "  Citadel Bootstrap  ·  v3.2 Stable"
echo

ERRORS=0

# ── 1. Prerequisites ──────────────────────────────────────────────────────────
section "Prerequisites"

check_cmd() {
    if command -v "$1" &>/dev/null; then
        ok "$1 found ($(command -v "$1"))"
    else
        fail "$1 not found — $2"
        ERRORS=$((ERRORS + 1))
    fi
}

check_cmd "cargo"      "install Rust: https://rustup.rs"
check_cmd "docker"     "install Docker Desktop (Windows) or Docker Engine (Linux)"
check_cmd "sqlite3"    "run: sudo apt-get install -y sqlite3"
check_cmd "protoc"     "run: sudo apt-get install -y protobuf-compiler"
check_cmd "git"        "install git"

if [[ $ERRORS -gt 0 ]]; then
    fail "$ERRORS prerequisite(s) missing. Please fix and re-run."
    exit 1
fi

# ── 2. Configuration Sync ─────────────────────────────────────────────────────
section "Configuration"

sync_config() {
    local src="$1"
    local dest="$2"
    if [[ ! -f "$dest" ]]; then
        cp "$src" "$dest"
        ok "$dest created from $(basename "$src")"
    else
        ok "$dest (already exists)"
    fi
}

sync_config "$KOAD_HOME/.env.template" "$KOAD_HOME/.env"
sync_config "$KOAD_HOME/config/defaults/kernel.toml" "$KOAD_HOME/config/kernel.toml"

# ── 3. Directories ────────────────────────────────────────────────────────────
section "Sanctuary Scaffolding"

DIRS=(
    "$BIN_DIR" "$LOG_DIR" "$KOAD_HOME/cache" "$KOAD_HOME/run"
    "$KOAD_HOME/data/db" "$KOAD_HOME/data/redis"
    "$KOAD_HOME/agents/bays" "$KOAD_HOME/agents/crews"
    "$KOAD_HOME/config/identities" "$KOAD_HOME/config/interfaces"
)

for dir in "${DIRS[@]}"; do
    mkdir -p "$dir"
done
ok "All sanctuary directories scaffolded."

# ── 4. Build Binaries ─────────────────────────────────────────────────────────
section "Build System"

PROTOC_BIN="${PROTOC:-$(command -v protoc)}"
PROTOC_INC="${PROTOC_INCLUDE:-$(dirname "$(command -v protoc)")/../include}"

# Fallback for common local installs
[[ ! -x "$PROTOC_BIN" && -x "$HOME/.local/bin/protoc" ]] && PROTOC_BIN="$HOME/.local/bin/protoc" && PROTOC_INC="$HOME/.local/include"

info "Building KoadOS Core binaries..."
PROTOC="$PROTOC_BIN" PROTOC_INCLUDE="$PROTOC_INC" \
    cargo build --manifest-path "$KOAD_HOME/Cargo.toml" \
    --bin koad --bin koad-agent --bin koad-citadel --bin koad-cass \
    --quiet

for bin in koad koad-agent koad-citadel koad-cass; do
    cp "$KOAD_HOME/target/debug/$bin" "$BIN_DIR/"
    ok "$bin → bin/"
done

# ── 5. DB Initialization ──────────────────────────────────────────────────────
section "Data Plane"

KOAD_DB="$KOAD_HOME/data/db/koad.db"
if [[ ! -f "$KOAD_DB" ]]; then
    bash "$KOAD_HOME/scripts/init-koad-db.sh"
    sqlite3 "$KOAD_DB" < "$KOAD_HOME/scripts/init-jupiter-db.sql"
    ok "Master records (SQLite) initialized."
else
    ok "Master records (SQLite) already exist."
fi

# ── 6. Shell Integration ──────────────────────────────────────────────────────
section "Shell Integration"

BASHRC="$HOME/.bashrc"
SENTINEL_START="# >>> KoadOS Initialize >>>"
SENTINEL_END="# <<< KoadOS Initialize <<<"

# We use the absolute path of the current installation
INTEGRATION_BLOCK="
$SENTINEL_START
# KoadOS Environment & Functions
export KOADOS_HOME=\"$KOAD_HOME\"
export PATH=\"\$KOADOS_HOME/bin:\$PATH\"
[ -f \"\$KOADOS_HOME/bin/koad-functions.sh\" ] && source \"\$KOADOS_HOME/bin/koad-functions.sh\"
$SENTINEL_END"

# Cleanup: Remove legacy KoadOS path entries that aren't inside our sentinel block
if grep -q "koad-os/bin" "$BASHRC"; then
    sed -i '/koad-os\/bin/ { /KoadOS Initialize/! d }' "$BASHRC"
fi

if grep -qF "$SENTINEL_START" "$BASHRC" 2>/dev/null; then
    # Update existing block
    sed -i "/$SENTINEL_START/,/$SENTINEL_END/d" "$BASHRC"
    echo "$INTEGRATION_BLOCK" >> "$BASHRC"
    ok "KoadOS block updated in $BASHRC"
else
    echo -e "\n$INTEGRATION_BLOCK" >> "$BASHRC"
    ok "KoadOS block added to $BASHRC"
fi

# ── 7. Verification ───────────────────────────────────────────────────────────
section "Verification"

export PATH="$BIN_DIR:$PATH"
export KOADOS_HOME="$KOAD_HOME"

if koad status &>/dev/null; then
    ok "Neural Link: ACTIVE"
else
    warn "Neural Link: DARK (expected - run 'koad system start' to ignite)"
fi

echo -e "\n${BOLD}${GREEN}Bootstrap complete.${RESET}"
echo "Next step: Run 'source ~/.bashrc' and then 'koad system start'."
echo
