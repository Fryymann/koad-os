#!/bin/bash
echo ">>> [DECK] Initiating Debris Sweep..."
rm -f *.log *.bak dump.rdb
cargo clean
rm -rf /tmp/pytest-of-ideans/ /tmp/koad_test_*.sock
echo ">>> [DECK] Hull cleared. Debris purged."
