#!/bin/sh

OUTPUT="$1"
INTERVAL="$2"

if [ -z "$OUTPUT" ]; then
    echo "Argument to print must be supplied!" &1>2
    exit 1
fi

if [ -z "$INTERVAL" ]; then
    INTERVAL=1
fi

while true; do
    echo "$OUTPUT"
    sleep "$INTERVAL"
done
