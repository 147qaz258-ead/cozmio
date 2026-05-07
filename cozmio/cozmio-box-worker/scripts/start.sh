#!/bin/bash
# Start cozmio-box-worker

WORKER_BIN="/opt/cozmio/bin/box-worker"
LOG_FILE="/opt/cozmio/logs/box-worker.log"
PID_FILE="/opt/cozmio/cozmio-box-worker.pid"

echo "Starting cozmio-box-worker..."

# Check if already running
if pgrep -f "cozmio-box-worker" > /dev/null; then
    echo "cozmio-box-worker is already running"
    exit 1
fi

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")"

# Start the worker
nohup "$WORKER_BIN" > "$LOG_FILE" 2>&1 &
WORKER_PID=$!

echo "cozmio-box-worker started (PID: $WORKER_PID)"
echo "$WORKER_PID" > "$PID_FILE"
