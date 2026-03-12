#!/bin/bash
set -x

cd /home/ideans/.koad-os

pkill -9 kspine || true
pkill -9 redis-server || true
rm -f koad.sock redis.pid

echo "Starting first spine..."
bin/kspine > spine_test.log 2>&1 &
SPINE_PID=$!
sleep 3

echo "Crashing Redis..."
if [ -f redis.pid ]; then
    kill -9 $(cat redis.pid)
else
    echo "NO REDIS PID FOUND!"
    pkill -9 redis-server
fi
sleep 1

echo "Socket status:"
ls -la koad.sock

echo "Starting recovery spine..."
bin/kspine > spine_recovery.log 2>&1 &
sleep 3

pkill -9 kspine || true
pkill -9 redis-server || true

echo "Recovery Log Contents:"
cat spine_recovery.log
