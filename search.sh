#!/bin/bash
set -o pipefail

if [ $# -ne 1 ]; then
    echo "USAGE" >&2
    echo "$0 <levels>" >&2
    exit 1
fi

levels=$1
givens=81
timeout=5
while true; do
    flags="--givens=$givens --max_inference_levels=$levels --timeout_seconds=$timeout"
    echo "Generating with $flags"
    output="$(cargo -q run --release -- $flags | tail -n 13)"
    if [ $? == 0 ]; then
        # Successfully generated a puzzle. Save it and try for one fewer given.
        best_puzzle="$(printf 'Found this puzzle with %s givens\n%s' $givens "$output")"
        givens=$((givens - 1))
    else
        # We went too far. Print the last successfully generated puzzle and give up.
        if [ -n "$best_puzzle" ]; then
            echo "$best_puzzle"
            exit 0
        else
            echo "Failed to generate a puzzle" >&2
            exit 1
        fi
    fi
done
