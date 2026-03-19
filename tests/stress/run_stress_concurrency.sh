#!/bin/bash
# Concurrency Stress: Run parallel boots to check for race conditions
COUNT=20
echo "Starting $COUNT parallel boots for agent 'tyr'..."

# Create a temporary directory for parallel results
TEMP_LOGS=$(mktemp -d)

# Use xargs to run parallel boots and capture exit codes
seq $COUNT | xargs -I {} -P $COUNT bash -c "agent-boot tyr > $TEMP_LOGS/boot_{}.log 2>&1; echo \$? > $TEMP_LOGS/exit_{}.log"

# Verify results
FAIL_COUNT=0
for i in $(seq $COUNT); do
  EXIT_CODE=$(cat $TEMP_LOGS/exit_$i.log)
  if [ "$EXIT_CODE" -ne 0 ]; then
    echo "Fail: Boot $i failed with exit code $EXIT_CODE"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
done

if [ "$FAIL_COUNT" -eq 0 ]; then
  echo "SUCCESS: All $COUNT parallel boots completed without error."
else
  echo "FAILURE: $FAIL_COUNT / $COUNT boots failed."
fi

# Cleanup
rm -rf $TEMP_LOGS
