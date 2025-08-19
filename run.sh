#!/bin/bash

PROCESS_NAME="rustcord"
SLEEP_DURATION_S=5

echo "Attempting to gracefully stop the '$PROCESS_NAME' process..."


PID=$(pgrep -f "$PROCESS_NAME")

if [ -n "$PID" ]; then
    echo "Found process with PID: $PID. Sending SIGTERM..."

    kill "$PID"

    sleep "$SLEEP_DURATION_S"

    if kill -0 "$PID" 2>/dev/null; then
        echo "Process '$PROCESS_NAME' did not exit gracefully. Sending SIGKILL..."
        kill -9 "$PID"
    else
        echo "Process '$PROCESS_NAME' exited successfully."
    fi
else
    echo "No running '$PROCESS_NAME' process found. Proceeding with startup."
fi

echo "Starting the new version of '$PROCESS_NAME'..."
bash -c "target/release/rustcord"