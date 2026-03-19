#!/bin/bash
# Hook-System Stress: Test resilient boot when hooks are oversized or slow

# Backup existing hook if any
HOOK_DIR="$HOME/.tyr/hooks"
HOOK_FILE="$HOOK_DIR/post_boot.sh"
BACKUP_HOOK="$HOOK_DIR/post_boot.sh.bak"

if [ -f "$HOOK_FILE" ]; then
    mv "$HOOK_FILE" "$BACKUP_HOOK"
fi

echo "Creating malicious hook (oversized output + delay)..."
cat << 'MALICIOUS' > "$HOOK_FILE"
#!/bin/bash
echo "Simulating a very slow and large hook..."
sleep 2
# Output 100KB of text
head -c 100000 /dev/zero | tr '\0' 'x'
echo -e "\n[DONE]"
MALICIOUS
chmod +x "$HOOK_FILE"

echo "Executing agent-boot with malicious hook..."
/usr/bin/time -v agent-boot tyr > /tmp/stress_hook_output.log 2>&1

# Check for resilience (did it hang indefinitely?)
# Assuming we expect a timeout around 3 seconds or it should at least finish.
ELAPSED=$(grep -a "Elapsed (wall clock) time" /tmp/stress_hook_output.log | awk '{print $NF}')
echo "Wall Clock Time: $ELAPSED"

# Cleanup
mv "$BACKUP_HOOK" "$HOOK_FILE"
echo "Hook-System test complete. Results captured in /tmp/stress_hook_output.log"
