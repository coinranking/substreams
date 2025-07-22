#!/bin/bash

set -e

# Default values
START_BLOCK=22964000
STOP_BLOCK=22965000
TOKEN=""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Usage function
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -s, --start-block BLOCK    Start block (default: $START_BLOCK)"
    echo "  -e, --stop-block BLOCK     Stop block (default: $STOP_BLOCK)"
    echo "  -t, --token TOKEN          Authorization token (required if not in env)"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "This script measures the performance of the Uniswap V3 substreams"
    echo "by tracking tickers per second during execution."
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
        export $(grep -E '^SUBSTREAMS_TOKEN=' "$ENV_FILE" | xargs)
        TOKEN="$SUBSTREAMS_TOKEN"
    fi
    
    if [ -z "$TOKEN" ]; then
        echo "Error: No token provided. Please either:"
        echo "  1. Set SUBSTREAMS_TOKEN in .env file"
        echo "  2. Pass token with -t option"
        exit 1
    fi
fi

echo -e "${BLUE}=== Uniswap V3 Substreams Performance Test ===${NC}"
echo -e "${YELLOW}Block range: $START_BLOCK to $STOP_BLOCK${NC}"
echo ""

# Build first
echo -e "${YELLOW}Building Substream...${NC}"
source ~/.cargo/env
cd $(dirname "$0")
cargo build --target wasm32-unknown-unknown --release --quiet

# Create a temporary file for output
TEMP_FILE=$(mktemp)
STATS_FILE=$(mktemp)

# Function to count tickers and blocks
count_stats() {
    local file=$1
    local ticker_count=$(grep -c '"poolAddress":' "$file" 2>/dev/null || echo 0)
    local block_count=$(grep -E '"@block":|^Progress:' "$file" | grep -E -o '[0-9]{8}' | sort -u | wc -l | tr -d ' ')
    echo "$ticker_count $block_count"
}

# Start the substreams command in background
echo -e "${YELLOW}Starting Substreams...${NC}"
echo ""

# Run substreams and capture output
substreams run -e mainnet.eth.streamingfast.io:443 \
    --header "Authorization: Bearer $TOKEN" \
    substreams.yaml \
    map_uniswap_ticker_output \
    --start-block $START_BLOCK \
    --stop-block $STOP_BLOCK \
    --production-mode=false \
    --limit-processed-blocks=0 \
    --output=json 2>&1 | tee "$TEMP_FILE" | \
while IFS= read -r line; do
    # Display progress lines
    if [[ "$line" =~ ^Progress: ]] || [[ "$line" =~ "TraceID" ]] || [[ "$line" =~ "Blocks to process" ]]; then
        echo "$line"
    fi
    
    # Count stats periodically
    if [[ "$line" =~ "@block" ]]; then
        echo "$line" >> "$STATS_FILE"
        
        # Every 10 blocks, show performance stats
        block_count=$(grep -c "@block" "$STATS_FILE" 2>/dev/null || echo 0)
        if [ $((block_count % 10)) -eq 0 ] && [ $block_count -gt 0 ]; then
            # Count total tickers so far
            total_tickers=$(grep -c '"poolAddress":' "$STATS_FILE" 2>/dev/null || echo 0)
            
            # Calculate elapsed time (approximate based on block count)
            # Assuming ~12 seconds per block on Ethereum
            elapsed_seconds=$((block_count * 12))
            
            # Calculate rate
            if [ $elapsed_seconds -gt 0 ]; then
                tickers_per_second=$(echo "scale=2; $total_tickers / $elapsed_seconds" | bc -l 2>/dev/null || echo "0")
                
                echo -e "\n${GREEN}Performance Stats:${NC}"
                echo -e "  Blocks processed: ${BLUE}$block_count${NC}"
                echo -e "  Total tickers: ${BLUE}$total_tickers${NC}"
                echo -e "  Tickers/second: ${GREEN}$tickers_per_second${NC}"
                echo -e "  Avg tickers/block: ${GREEN}$(echo "scale=2; $total_tickers / $block_count" | bc -l 2>/dev/null || echo "0")${NC}\n"
            fi
        fi
    fi
done

# Final statistics
echo -e "\n${GREEN}=== Final Performance Report ===${NC}"

# Count final stats
final_stats=$(count_stats "$TEMP_FILE")
total_tickers=$(echo $final_stats | cut -d' ' -f1)
total_blocks=$(echo $final_stats | cut -d' ' -f2)

# Calculate block range
block_range=$((STOP_BLOCK - START_BLOCK))

# Get actual runtime from the output if available
runtime=$(grep -E "took [0-9]+\.[0-9]+s" "$TEMP_FILE" | tail -1 | grep -o '[0-9]\+\.[0-9]\+s' || echo "")

echo -e "${YELLOW}Summary:${NC}"
echo -e "  Block range: ${BLUE}$START_BLOCK - $STOP_BLOCK${NC} ($block_range blocks)"
echo -e "  Total blocks processed: ${BLUE}$total_blocks${NC}"
echo -e "  Total tickers generated: ${BLUE}$total_tickers${NC}"

if [ $total_blocks -gt 0 ]; then
    avg_tickers=$(echo "scale=2; $total_tickers / $total_blocks" | bc -l 2>/dev/null || echo "0")
    echo -e "  Average tickers per block: ${GREEN}$avg_tickers${NC}"
fi

if [ -n "$runtime" ]; then
    # Extract numeric value from runtime
    runtime_seconds=$(echo "$runtime" | sed 's/s$//')
    if [ -n "$runtime_seconds" ] && [ "$runtime_seconds" != "0" ]; then
        tickers_per_second=$(echo "scale=2; $total_tickers / $runtime_seconds" | bc -l 2>/dev/null || echo "0")
        blocks_per_second=$(echo "scale=2; $total_blocks / $runtime_seconds" | bc -l 2>/dev/null || echo "0")
        
        echo -e "\n${YELLOW}Performance Metrics:${NC}"
        echo -e "  Runtime: ${BLUE}$runtime${NC}"
        echo -e "  Tickers per second: ${GREEN}$tickers_per_second${NC}"
        echo -e "  Blocks per second: ${GREEN}$blocks_per_second${NC}"
    fi
fi

# Show top pools by activity
echo -e "\n${YELLOW}Top 5 Most Active Pools:${NC}"
grep -o '"poolAddress":"[^"]*"' "$TEMP_FILE" | sort | uniq -c | sort -nr | head -5 | \
while read count pool; do
    pool_addr=$(echo $pool | cut -d'"' -f4)
    echo -e "  ${BLUE}$pool_addr${NC}: $count tickers"
done

# Cleanup
rm -f "$TEMP_FILE" "$STATS_FILE"

echo -e "\n${GREEN}Performance test completed!${NC}"