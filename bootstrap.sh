#!/usr/bin/env bash
# =============================================================================
# KoadOS Bootstrap — First-time setup after `git clone`
# Run from the repo root: bash bootstrap.sh
# =============================================================================
set -euo pipefail

KOAD_HOME="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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

echo -e "${BOLD}"
echo "  ██╗  ██╗ ██████╗  █████╗ ██████╗      ██████╗ ███████╗"
echo "  ██║ ██╔╝██╔═══██╗██╔══██╗██╔══██╗    ██╔═══██╗██╔════╝"
echo "  █████╔╝ ██║   ██║███████║██║  ██║    ██║   ██║███████╗"
echo "  ██╔═██╗ ██║   ██║██╔══██║██║  ██║    ██║   ██║╚════██║"
echo "  ██║  ██╗╚██████╔╝██║  ██║██████╔╝    ╚██████╔╝███████║"
echo "  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝      ╚═════╝ ╚══════╝"
echo -e "${RESET}"
echo "  Citadel Bootstrap  ·  v3.2"
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
check_cmd "docker"     && docker compose version &>/dev/null \
    && ok "docker compose available" \
    || { fail "docker compose not available"; ERRORS=$((ERRORS + 1)); }
check_cmd "sqlite3"    "run: sudo apt-get install -y sqlite3"
check_cmd "protoc"     "run: sudo apt-get install -y protobuf-compiler  OR install to ~/.local/bin/protoc"
check_cmd "git"        "install git"

if [[ $ERRORS -gt 0 ]]; then
    echo
    fail "$ERRORS prerequisite(s) missing. Fix the above and re-run bootstrap.sh."
    exit 1
fi

# ── 2. Environment file ───────────────────────────────────────────────────────
section "Environment"

if [[ -f "$KOAD_HOME/.env" ]]; then
    ok ".env already exists — skipping template copy"
else
    cp "$KOAD_HOME/.env.template" "$KOAD_HOME/.env"
    ok ".env created from template"
    warn "Open $KOAD_HOME/.env and populate your secrets before running \`koad system init\`"
fi

# ── 3. Directories ────────────────────────────────────────────────────────────
section "Directories"

for dir in "$BIN_DIR" "$LOG_DIR" \
    "$KOAD_HOME/cache" \
    "$KOAD_HOME/data/redis" \
    "$KOAD_HOME/.agents"; do
    mkdir -p "$dir"
    ok "$dir"
done

# ── 4. Script permissions ─────────────────────────────────────────────────────
section "Script Permissions"

find "$KOAD_HOME/scripts" -name "*.sh" -exec chmod +x {} \;
ok "scripts/*.sh marked executable"

# ── 5. Build binaries ─────────────────────────────────────────────────────────
section "Build (this may take a few minutes)"

PROTOC_BIN="${PROTOC:-$(command -v protoc)}"
PROTOC_INC="${PROTOC_INCLUDE:-$(dirname "$(command -v protoc)")/../include}"

# Try ~/.local/bin/protoc fallback (KoadOS standard location)
if [[ ! -x "$PROTOC_BIN" ]] && [[ -x "$HOME/.local/bin/protoc" ]]; then
    PROTOC_BIN="$HOME/.local/bin/protoc"
    PROTOC_INC="$HOME/.local/include"
fi

info "protoc: $PROTOC_BIN"
info "Building all binaries (koad, koad-agent, koad-citadel, koad-cass)..."

PROTOC="$PROTOC_BIN" PROTOC_INCLUDE="$PROTOC_INC" \
    cargo build --manifest-path "$KOAD_HOME/Cargo.toml" \
    --bin koad --bin koad-agent --bin koad-citadel --bin koad-cass \
    2>&1 | grep -E "^error|Compiling koad|Finished|warning.*unused" || true

for bin in koad koad-agent koad-citadel koad-cass; do
    src="$KOAD_HOME/target/debug/$bin"
    if [[ -x "$src" ]]; then
        cp "$src" "$BIN_DIR/$bin"
        ok "$bin → $BIN_DIR/"
    else
        fail "$bin binary not found after build"
        ERRORS=$((ERRORS + 1))
    fi
done

[[ $ERRORS -gt 0 ]] && { fail "Build failed. Check cargo output above."; exit 1; }

# ── 6. Docker stack ───────────────────────────────────────────────────────────
section "Docker Stack"

if docker info &>/dev/null; then
    info "Starting Redis Stack + Qdrant..."
    docker compose -f "$KOAD_HOME/docker-compose.yml" up -d 2>&1 | grep -v "^#" || true

    # Brief settle time for containers to bind ports
    sleep 2

    if docker ps --format '{{.Names}}' | grep -q "koad-redis-stack"; then
        ok "koad-redis-stack running"
    else
        warn "koad-redis-stack not detected — check: docker compose logs citadel-redis"
    fi
    if docker ps --format '{{.Names}}' | grep -q "koad-qdrant"; then
        ok "koad-qdrant running"
    else
        warn "koad-qdrant not detected — check: docker compose logs qdrant"
    fi
else
    warn "Docker daemon not reachable. Start Docker Desktop and re-run \`koad system init\` after bootstrap."
fi

# ── 7. SQLite databases ───────────────────────────────────────────────────────
section "SQLite Databases"

KOAD_DB="$KOAD_HOME/koad.db"

bash "$KOAD_HOME/scripts/init-koad-db.sh"
ok "koad.db — core schema initialised"

sqlite3 "$KOAD_DB" < "$KOAD_HOME/scripts/init-jupiter-db.sql"
ok "koad.db — WAL + episodic/procedural tables initialised"

WAL=$(sqlite3 "$KOAD_DB" "PRAGMA journal_mode;" 2>/dev/null)
[[ "$WAL" == "wal" ]] && ok "WAL mode confirmed" || warn "WAL mode not confirmed (got: $WAL)"

# ── 8. Shell integration ──────────────────────────────────────────────────────
section "Shell Integration"

BASHRC="$HOME/.bashrc"
SOURCE_LINE="[ -f \"\$HOME/.koad-os/bin/koad-functions.sh\" ] && source \"\$HOME/.koad-os/bin/koad-functions.sh\""

if grep -qF "koad-functions.sh" "$BASHRC" 2>/dev/null; then
    ok "koad-functions.sh already sourced in $BASHRC"
else
    echo "" >> "$BASHRC"
    echo "# KoadOS Shell Functions" >> "$BASHRC"
    echo "$SOURCE_LINE" >> "$BASHRC"
    ok "Added koad-functions.sh source to $BASHRC"
fi

PATH_LINE="[ -d \"\$HOME/.koad-os/bin\" ] && PATH=\"\$HOME/.koad-os/bin:\$PATH\""
if grep -qF ".koad-os/bin" "$BASHRC" 2>/dev/null; then
    ok "~/.koad-os/bin already in PATH via $BASHRC"
else
    echo "$PATH_LINE" >> "$BASHRC"
    ok "Added ~/.koad-os/bin to PATH in $BASHRC"
fi

# ── 9. Verify ─────────────────────────────────────────────────────────────────
section "Verification"

export PATH="$BIN_DIR:$PATH"
KOADOS_HOME="$KOAD_HOME" "$BIN_DIR/koad" doctor 2>&1 || true

# ── Done ──────────────────────────────────────────────────────────────────────
echo
echo -e "${BOLD}${GREEN}Bootstrap complete.${RESET}"
echo
echo "  Next steps:"
echo "  1. Fill in your secrets:  \$EDITOR $KOAD_HOME/.env"
echo "  2. Reload your shell:     source ~/.bashrc"
echo "  3. Finalize environment:  koad system init"
echo "  4. Boot an agent:         agent-boot tyr"
echo
