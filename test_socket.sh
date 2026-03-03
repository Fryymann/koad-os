#!/bin/bash
set -e

cd /home/ideans/.koad-os

echo "1. Building Spine..."
cargo build --release -p koad-spine
cp target/release/koad-spine bin/kspine

echo "2. Ensuring clean slate..."
pkill -9 kspine || true
pkill -9 redis-server || true
rm -f koad.sock redis.pid

echo "3. Starting Spine (will start Redis)..."
bin/kspine &
SPINE_PID=$!
sleep 5

echo "4. Simulating Crash (Leaving ghost socket)..."
kill -9 $SPINE_PID
# Soft kill to Redis to see if it cleans up, wait no, hard kill to FORCE it to leave the socket.
kill -9 $(cat redis.pid) || pkill -9 redis-server || true

echo "Socket status after crash:"
ls -la koad.sock || echo "SOCKET DELETED UNEXPECTEDLY"

echo "5. Attempting Recovery Boot..."
bin/kspine &
REC_PID=$!
sleep 5

echo "6. Cleanup..."
kill -9 $REC_PID
pkill -9 redis-server || true
echo "Test Complete."
