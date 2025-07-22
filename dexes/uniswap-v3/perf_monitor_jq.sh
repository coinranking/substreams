#!/bin/bash

set -e

# Default values
START_BLOCK=22964000
STOP_BLOCK=22964100
TOKEN=""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -s, --start-block BLOCK    Start block (default: $START_BLOCK)"
    echo "  -e, --stop-block BLOCK     Stop block (default: $STOP_BLOCK)"
    echo "  -t, --token TOKEN          Authorization token"
    echo "  -h, --help                 Show this help message"
    echo ""
    echo "Real-time performance monitor using jq for JSON parsing"
    exit 1
}

# Check for jq
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed${NC}"
    echo "Install with: brew install jq"
    exit 1
fi

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--start-block) START_BLOCK="$2"; shift 2 ;;
        -e|--stop-block) STOP_BLOCK="$2"; shift 2 ;;
        -t|--token) TOKEN="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

# Load token
if [ -z "$TOKEN" ]; then
    ENV_FILE="$(dirname "$0")/../../.env"
    if [ -f "$ENV_FILE" ]; then
        export $(grep -E '^SUBSTREAMS_TOKEN=' "$ENV_FILE" | xargs)
        TOKEN="$SUBSTREAMS_TOKEN"
    fi
    
    if [ -z "$TOKEN" ]; then
        echo -e "${RED}Error: No token provided${NC}"
        exit 1
    fi
fi

# Build
echo -e "${YELLOW}Building...${NC}"
source ~/.cargo/env
cd $(dirname "$0")
cargo build --target wasm32-unknown-unknown --release --quiet

# Create temp files
JSON_BUFFER=$(mktemp)
STATS_FILE=$(mktemp)

# Cleanup on exit
trap "rm -f $JSON_BUFFER $STATS_FILE" EXIT

# Start time
START_TIME=$(date +%s)

echo -e "${GREEN}=== Performance Monitor (jq version) ===${NC}"
echo -e "Blocks: ${BLUE}$START_BLOCK - $STOP_BLOCK${NC}"
echo ""

# Header
printf "%-12s %-10s %-12s %-12s %-15s\n" "Time(s)" "Block#" "Tickers" "Total" "Avg/Block"
echo "---------------------------------------------------------------"

# Initialize
TOTAL_BLOCKS=0
TOTAL_TICKERS=0

# Process function
process_json_block() {
    local json="$1"
    
    # Extract block number and ticker count using jq
    local block_num=$(echo "$json" | jq -r '."@block"' 2>/dev/null)
    local ticker_count=$(echo "$json" | jq -r '."@data".tickers | length' 2>/dev/null)
    
    if [ -n "$block_num" ] && [ "$block_num" != "null" ] && [ -n "$ticker_count" ]; then
        ((TOTAL_BLOCKS++))
        TOTAL_TICKERS=$((TOTAL_TICKERS + ticker_count))
        
        # Save to stats
        echo "$block_num $ticker_count" >> "$STATS_FILE"
        
        # Calculate elapsed time
        CURRENT_TIME=$(date +%s)
        ELAPSED=$((CURRENT_TIME - START_TIME))
        
        # Calculate average
        AVG_TICKERS=$(echo "scale=2; $TOTAL_TICKERS / $TOTAL_BLOCKS" | bc -l 2>/dev/null || echo "0")
        
        # Display
        printf "%-12s %-10s %-12s %-12s %-15s\n" \
            "$ELAPSED" \
            "$block_num" \
            "$ticker_count" \
            "$TOTAL_TICKERS" \
            "$AVG_TICKERS"
    fi
}

# Run substreams
echo -e "${YELLOW}Running substreams...${NC}\n"

# Buffer for accumulating JSON
JSON_ACC=""
BRACE_COUNT=0

./test.sh -s $START_BLOCK -e $STOP_BLOCK -o json 2>&1 | \
while IFS= read -r line; do
    # Skip non-JSON lines
    if [[ "$line" =~ ^(Building|Running|TraceID|Progress|Blocks|Error|Hint) ]]; then
        echo -e "${YELLOW}$line${NC}" >&2
        continue
    fi
    
    # Accumulate JSON
    if [[ "$line" =~ ^\{ ]]; then
        JSON_ACC="$line"
        BRACE_COUNT=1
    elif [ -n "$JSON_ACC" ]; then
        JSON_ACC="${JSON_ACC}
${line}"
        
        # Count braces
        OPEN=$(echo "$line" | grep -o '{' | wc -l)
        CLOSE=$(echo "$line" | grep -o '}' | wc -l)
        BRACE_COUNT=$((BRACE_COUNT + OPEN - CLOSE))
        
        # Complete JSON object
        if [ $BRACE_COUNT -eq 0 ]; then
            process_json_block "$JSON_ACC"
            JSON_ACC=""
        fi
    fi
done

# Final statistics
echo -e "\n${GREEN}=== Final Performance Report ===${NC}"

FINAL_TIME=$(date +%s)
TOTAL_ELAPSED=$((FINAL_TIME - START_TIME))

if [ $TOTAL_ELAPSED -gt 0 ] && [ $TOTAL_BLOCKS -gt 0 ]; then
    FINAL_TPS=$(echo "scale=2; $TOTAL_TICKERS / $TOTAL_ELAPSED" | bc -l)
    FINAL_BPS=$(echo "scale=2; $TOTAL_BLOCKS / $TOTAL_ELAPSED" | bc -l)
    FINAL_TPB=$(echo "scale=2; $TOTAL_TICKERS / $TOTAL_BLOCKS" | bc -l)
    
    echo -e "Runtime: ${BLUE}${TOTAL_ELAPSED}s${NC}"
    echo -e "Blocks: ${BLUE}$TOTAL_BLOCKS${NC}"
    echo -e "Tickers: ${BLUE}$TOTAL_TICKERS${NC}"
    echo -e "Performance: ${GREEN}$FINAL_TPS tickers/s${NC}, ${GREEN}$FINAL_BPS blocks/s${NC}"
    echo -e "Average: ${GREEN}$FINAL_TPB tickers/block${NC}"
    
    # Distribution
    echo -e "\n${YELLOW}Activity Distribution:${NC}"
    ZERO_BLOCKS=$(grep " 0$" "$STATS_FILE" 2>/dev/null | wc -l | tr -d ' ')
    ACTIVE_BLOCKS=$(grep -v " 0$" "$STATS_FILE" 2>/dev/null | wc -l | tr -d ' ')
    echo "Blocks with swaps: $ACTIVE_BLOCKS"
    echo "Blocks without swaps: $ZERO_BLOCKS"
    
    # Most active blocks
    if [ -s "$STATS_FILE" ]; then
        echo -e "\n${YELLOW}Most Active Blocks:${NC}"
        sort -k2 -nr "$STATS_FILE" | head -5 | while read block count; do
            if [ $count -gt 0 ]; then
                echo -e "  Block ${BLUE}$block${NC}: $count tickers"
            fi
        done
    fi
fi