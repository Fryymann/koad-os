#!/usr/bin/env bash
# rook-up.sh — Bootstrap Rook (Claude Desktop CASS Memory Bridge)
# Run once per machine to stand up the local memory stack.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/data"

echo "[rook] Standing up CASS memory stack..."

# Create data directories
mkdir -p "$DATA_DIR/redis" "$DATA_DIR/qdrant" "$DATA_DIR/sqlite"

# Pull images
echo "[rook] Pulling images..."
docker compose -f "$SCRIPT_DIR/docker-compose.yml" pull

# Start stack
echo "[rook] Starting services..."
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d

# Wait for MCP to be ready
echo "[rook] Waiting for MCP server..."
for i in $(seq 1 20); do
    if curl -sf http://localhost:9742/health >/dev/null 2>&1; then
        echo "[rook] MCP server ready."
        break
    fi
    sleep 2
    if [ "$i" -eq 20 ]; then
        echo "[rook] WARNING: MCP server did not respond after 40s. Check: docker compose -f $SCRIPT_DIR/docker-compose.yml logs koad-os-mcp"
    fi
done

echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  Rook is online. Add this to claude_desktop_config.json:   │"
echo "│                                                             │"
echo "│  \"mcpServers\": {                                           │"
echo "│    \"rook\": {                                               │"
echo "│      \"transport\": \"http\",                                  │"
echo "│      \"url\": \"http://localhost:9742/mcp\"                    │"
echo "│    }                                                        │"
echo "│  }                                                          │"
echo "└─────────────────────────────────────────────────────────────┘"
