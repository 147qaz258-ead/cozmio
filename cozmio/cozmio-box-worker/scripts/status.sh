#!/bin/bash
# Check status of cozmio-box-worker

PID_FILE="/opt/cozmio/cozmio-box-worker.pid"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "cozmio-box-worker is running (PID: $PID)"
        exit 0
    else
        echo "cozmio-box-worker PID file exists but process is not running"
        exit 1
    fi
else
    # Try to find by process name
    PIDS=$(pgrep -f "cozmio-box-worker")
    if [ -n "$PIDS" ]; then
        echo "cozmio-box-worker is running (PIDs: $PIDS)"
        exit 0
    else
        echo "cozmio-box-worker is not running"
        exit 1
    fi
fi
