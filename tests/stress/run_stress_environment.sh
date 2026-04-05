#!/bin/bash
# Environmental Stress: Simulate CASS/Citadel downtime

echo "Starting environmental failure stress test..."

# Backup current kernel.toml if it exists
KOAD_HOME="${KOAD_HOME:-$HOME/.koad-os}"
CONFIG_PATH="$KOAD_HOME/config/kernel.toml"
BACKUP_PATH="$KOAD_HOME/config/kernel.toml.bak"

if [ -f "$CONFIG_PATH" ]; then
    cp "$CONFIG_PATH" "$BACKUP_PATH"
    # Point gRPC to a non-existent port
    sed -i 's/grpc_addr = "[^"]*"/grpc_addr = "127.0.0.1:9999"/g' "$CONFIG_PATH"
fi

# Run boot and time it
echo "Running boot with mocked offline backend..."
time agent-boot tyr > /tmp/stress_env_output.log 2>&1

# Check if it failed gracefully and within time
BOOT_STATUS=$(grep -a "Status:" /tmp/stress_env_output.log)
echo "Captured Boot Status: $BOOT_STATUS"

if grep -q "FAIL" <<< "$BOOT_STATUS" || grep -q "ERROR" <<< "$BOOT_STATUS"; then
    echo "SUCCESS: System handled backend downtime gracefully."
else
    echo "WARNING: System did not report expected failure status (Check logs)."
fi

# Restore backup
if [ -f "$BACKUP_PATH" ]; then
    mv "$BACKUP_PATH" "$CONFIG_PATH"
fi
