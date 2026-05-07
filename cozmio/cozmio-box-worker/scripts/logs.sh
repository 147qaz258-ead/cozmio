#!/bin/bash
# Show recent logs from cozmio-box-worker

LOG_FILE="/opt/cozmio/logs/box-worker.log"
LINES=${1:-50}

if [ -f "$LOG_FILE" ]; then
    tail -n "$LINES" "$LOG_FILE"
else
    echo "Log file not found: $LOG_FILE"
    exit 1
fi
