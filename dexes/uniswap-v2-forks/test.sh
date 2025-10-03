#!/bin/bash

set -e

# Default values
# Network-specific defaults (can be overridden with CLI args)
# Use --start-block to specify the appropriate block for your target network
START_BLOCK=10000835  # Default: Uniswap V2 deployment on Ethereum
STOP_BLOCK=10000935   # 100 blocks after START_BLOCK
OUTPUT_FORMAT="json"
FILTER_OUTPUT=false
TOKEN=""

# Usage function
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -s, --start-block BLOCK    Start block (default: $START_BLOCK)"
    echo "  -e, --stop-block BLOCK     Stop block (default: $STOP_BLOCK)"
    echo "  -f, --filter               Filter output to show only blocks with swaps"
    echo "  -o, --output FORMAT        Output format: json, jsonl, ui (default: json)"
    echo "  -t, --token TOKEN          Authorization token (required if not in env)"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                         # Run with defaults"
    echo "  $0 -s 10000835 -e 10000850 -f  # Test swap activity with filtering"
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
    # Try to load from .env file
    ENV_FILE="$(dirname "$0")/../../.env"
    if [ -f "$ENV_FILE" ]; then
        # Load the token from .env file
        export $(grep -E '^SUBSTREAMS_TOKEN=' "$ENV_FILE" | xargs)
        TOKEN="$SUBSTREAMS_TOKEN"
    fi
    
    if [ -z "$TOKEN" ]; then
        echo "Error: No token provided. Please either:"
        echo "  1. Set SUBSTREAMS_TOKEN in .env file"
        echo "  2. Pass token with -t option"
        echo "  3. Set TOKEN environment variable"
        exit 1
    fi
fi

echo "Building V2 Substream..."
source ~/.cargo/env
cd $(dirname "$0")  # Ensure we're in the right directory
cargo build --target wasm32-unknown-unknown --release

echo "Running V2 Substream test..."
echo "Block range: $START_BLOCK to $STOP_BLOCK"
echo "Output format: $OUTPUT_FORMAT"

# Build the command
# Default to mainnet, can be overridden with ENDPOINT env var
ENDPOINT=${ENDPOINT:-"mainnet.eth.streamingfast.io:443"}
CMD="substreams run -e $ENDPOINT"
CMD="$CMD --header \"Authorization: Bearer $TOKEN\""
CMD="$CMD substreams.yaml"
CMD="$CMD map_v2_ticker_output"
CMD="$CMD --start-block $START_BLOCK"
CMD="$CMD --stop-block $STOP_BLOCK"
CMD="$CMD --production-mode=false"
CMD="$CMD --limit-processed-blocks=0"
CMD="$CMD --output=$OUTPUT_FORMAT"

# Run the command with optional filtering
if [ "$FILTER_OUTPUT" = true ] && [ "$OUTPUT_FORMAT" = "json" ]; then
    echo "Filtering output to show only blocks with swaps..."
    # First run the command and save output
    OUTPUT=$(eval "$CMD" 2>&1)
    # Check if it's JSON and filter, otherwise show raw output
    echo "$OUTPUT" | jq -r 'select(.data.map_v2_ticker_output != null) | .data.map_v2_ticker_output | if (.tickers | length) > 0 then . else empty end' 2>/dev/null || echo "$OUTPUT"
else
    eval "$CMD"
fi

echo "Test completed!"