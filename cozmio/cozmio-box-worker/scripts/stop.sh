#!/bin/bash
# Stop cozmio-box-worker

PID_FILE="/opt/cozmio/cozmio-box-worker.pid"

echo "Stopping cozmio-box-worker..."

# Check if PID file exists
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        kill "$PID"
        rm -f "$PID_FILE"
        echo "cozmio-box-worker stopped (PID: $PID)"
    else
        echo "Process $PID not found, removing stale PID file"
        rm -f "$PID_FILE"
    fi
else
    # Try to find by process name
    PIDS=$(pgrep -f "cozmio-box-worker")
    if [ -n "$PIDS" ]; then
        echo "$PIDS" | xargs kill 2>/dev/null
        echo "cozmio-box-worker processes stopped"
    else
        echo "cozmio-box-worker is not running"
    fi
fi
