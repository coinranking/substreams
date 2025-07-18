#!/bin/bash

set -e

# Default values
# Note: Ethereum produces ~7,200 blocks per day (1 block every ~12 seconds)
# IMPORTANT: START_BLOCK should match initialBlock in substreams.yaml for testing
# For production, set initialBlock to START_BLOCK - 7200 (24 hours earlier)
START_BLOCK=22939174  # Update this to a recent block for testing
STOP_BLOCK=22939679   # ~500 blocks after START_BLOCK
OUTPUT_FORMAT="json"
FILTER_OUTPUT=false
TOKEN=""

# Usage function
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -s, --start-block BLOCK    Start block (default: $START_BLOCK)"
    echo "  -e, --stop-block BLOCK     Stop block (default: $STOP_BLOCK)"
    echo "  -f, --filter               Filter output to show only blocks with swaps/rolling volumes"
    echo "  -o, --output FORMAT        Output format: json, jsonl, ui (default: json)"
    echo "  -t, --token TOKEN          Authorization token (required if not in env)"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                         # Run with defaults"
    echo "  $0 -s 12400000 -e 12400010 -f  # Test swap activity with filtering"
    echo "  $0 --output ui             # Use UI output format"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--start-block)
            START_BLOCK="$2"
            shift 2
            ;;
        -e|--stop-block)
            STOP_BLOCK="$2"
            shift 2
            ;;
        -f|--filter)
            FILTER_OUTPUT=true
            shift
            ;;
        -o|--output)
            OUTPUT_FORMAT="$2"
            shift 2
            ;;
        -t|--token)
            TOKEN="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Check for token
if [ -z "$TOKEN" ]; then
    # Use default token if none provided
    TOKEN="eyJhbGciOiJLTVNFUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3ODgxNDYzMjgsImp0aSI6Ijk4NmIwZTQ4LTRhMWUtNDBiMC1iNGMzLWExMGYwMzQ4NmViNCIsImlhdCI6MTc1MjE0NjMyOCwiaXNzIjoiZGZ1c2UuaW8iLCJzdWIiOiIwc2F3aTI2ODFlZjA0M2Y5YWY3NzEiLCJ2IjoxLCJha2kiOiIzOTQ5MzBmMTEzYTk5N2I5NzU2Mjc2OTlhNWRiODBhNDUzNDEyM2FlNGNjZTgxYWViY2ExNzljZTMyOTUyZjVmIiwidWlkIjoiMHNhd2kyNjgxZWYwNDNmOWFmNzcxIn0.LqaZb-AiX7Sg9quyArJo2SseGrmpFVo8IbEu5wuo2ZbJbrBMYxWoWeBoOOT2_N2gcq-r_w0x3V0VTzHvhSv72g"
fi

echo "Building Substream..."
source ~/.cargo/env
cargo build --target wasm32-unknown-unknown --release

echo "Running Substream test..."
echo "Block range: $START_BLOCK to $STOP_BLOCK"
echo "Output format: $OUTPUT_FORMAT"

# Build the command
CMD="substreams run -e mainnet.eth.streamingfast.io:443"
CMD="$CMD --header \"Authorization: Bearer $TOKEN\""
CMD="$CMD substreams.yaml"
CMD="$CMD map_uniswap_output"
CMD="$CMD --start-block $START_BLOCK"
CMD="$CMD --stop-block $STOP_BLOCK"
CMD="$CMD --production-mode=false"
CMD="$CMD --output=$OUTPUT_FORMAT"

# Run the command with optional filtering
if [ "$FILTER_OUTPUT" = true ] && [ "$OUTPUT_FORMAT" = "json" ]; then
    echo "Filtering output to show only blocks with swaps or rolling volumes..."
    # First run the command and save output
    OUTPUT=$(eval "$CMD" 2>&1)
    # Check if it's JSON and filter, otherwise show raw output
    echo "$OUTPUT" | jq -r 'select(.data.map_uniswap_output != null) | .data.map_uniswap_output | if ((.swaps | length) > 0 or (.rollingVolumes | length) > 0) then . else empty end' 2>/dev/null || echo "$OUTPUT"
else
    eval "$CMD"
fi

echo "Test completed!"
