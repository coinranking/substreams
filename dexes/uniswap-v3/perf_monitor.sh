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
    echo "Real-time performance monitor for Uniswap V3 substreams"
    exit 1
}

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
TEMP_OUTPUT=$(mktemp)
BLOCK_STATS=$(mktemp)

# Cleanup on exit
trap "rm -f $TEMP_OUTPUT $BLOCK_STATS" EXIT

# Start time
START_TIME=$(date +%s)

echo -e "${GREEN}=== Real-time Performance Monitor ===${NC}"
echo -e "Blocks: ${BLUE}$START_BLOCK - $STOP_BLOCK${NC}"
echo -e "Press Ctrl+C to stop\n"

# Header for stats
printf "%-10s %-10s %-15s %-15s %-15s\n" "Time(s)" "Blocks" "Tickers" "Tickers/s" "Tickers/Block"
echo "----------------------------------------------------------------"

# Initialize counters
TOTAL_BLOCKS=0
TOTAL_TICKERS=0
CURRENT_BLOCK=""
BLOCK_TICKERS=0
IN_BLOCK=false

# Run substreams and monitor
./test.sh -s $START_BLOCK -e $STOP_BLOCK -o json 2>&1 | \
while IFS= read -r line; do
    # Save output
    echo "$line" >> "$TEMP_OUTPUT"
    
    # Check for new block
    if [[ "$line" =~ \"@block\":\ ([0-9]+) ]]; then
        NEW_BLOCK="${BASH_REMATCH[1]}"
        
        # If we were tracking a previous block, save its stats
        if [ -n "$CURRENT_BLOCK" ] && [ "$IN_BLOCK" = true ]; then
            echo "$CURRENT_BLOCK $BLOCK_TICKERS" >> "$BLOCK_STATS"
            ((TOTAL_BLOCKS++))
            TOTAL_TICKERS=$((TOTAL_TICKERS + BLOCK_TICKERS))
            
            # Calculate and display metrics
            CURRENT_TIME=$(date +%s)
            ELAPSED=$((CURRENT_TIME - START_TIME))
            
            if [ $ELAPSED -gt 0 ]; then
                TICKERS_PER_SEC=$(echo "scale=2; $TOTAL_TICKERS / $ELAPSED" | bc -l 2>/dev/null || echo "0")
                AVG_TICKERS_BLOCK=$(echo "scale=2; $TOTAL_TICKERS / $TOTAL_BLOCKS" | bc -l 2>/dev/null || echo "0")
                
                # Display update
                printf "%-10s %-10s %-15s %-15s %-15s\n" \
                    "$ELAPSED" \
                    "$TOTAL_BLOCKS" \
                    "$TOTAL_TICKERS" \
                    "$TICKERS_PER_SEC" \
                    "$AVG_TICKERS_BLOCK"
            fi
        fi
        
        # Start tracking new block
        CURRENT_BLOCK="$NEW_BLOCK"
        BLOCK_TICKERS=0
        IN_BLOCK=true
    fi
    
    # Count tickers in current block
    if [ "$IN_BLOCK" = true ] && [[ "$line" =~ \"poolAddress\": ]]; then
        ((BLOCK_TICKERS++))
    fi
    
    # Progress messages
    if [[ "$line" =~ ^Progress: ]] || [[ "$line" =~ TraceID ]] || [[ "$line" =~ "Blocks to process" ]]; then
        echo -e "${YELLOW}$line${NC}" >&2
    fi
done

# Handle the last block
if [ -n "$CURRENT_BLOCK" ] && [ "$IN_BLOCK" = true ]; then
    echo "$CURRENT_BLOCK $BLOCK_TICKERS" >> "$BLOCK_STATS"
    ((TOTAL_BLOCKS++))
    TOTAL_TICKERS=$((TOTAL_TICKERS + BLOCK_TICKERS))
fi

echo -e "\n${GREEN}=== Final Statistics ===${NC}"

# Final calculations
FINAL_TIME=$(date +%s)
TOTAL_ELAPSED=$((FINAL_TIME - START_TIME))

if [ $TOTAL_ELAPSED -gt 0 ] && [ $TOTAL_BLOCKS -gt 0 ]; then
    FINAL_TPS=$(echo "scale=2; $TOTAL_TICKERS / $TOTAL_ELAPSED" | bc -l)
    FINAL_BPS=$(echo "scale=2; $TOTAL_BLOCKS / $TOTAL_ELAPSED" | bc -l)
    FINAL_TPB=$(echo "scale=2; $TOTAL_TICKERS / $TOTAL_BLOCKS" | bc -l)
    
    echo -e "Total runtime: ${BLUE}${TOTAL_ELAPSED}s${NC}"
    echo -e "Blocks processed: ${BLUE}$TOTAL_BLOCKS${NC}"
    echo -e "Tickers generated: ${BLUE}$TOTAL_TICKERS${NC}"
    echo -e "Performance: ${GREEN}$FINAL_TPS tickers/s${NC}, ${GREEN}$FINAL_BPS blocks/s${NC}"
    echo -e "Average: ${GREEN}$FINAL_TPB tickers/block${NC}"
    
    # Show distribution
    echo -e "\n${YELLOW}Block Distribution:${NC}"
    echo "Blocks with 0 tickers: $(grep " 0$" "$BLOCK_STATS" 2>/dev/null | wc -l | tr -d ' ')"
    echo "Blocks with tickers: $(grep -v " 0$" "$BLOCK_STATS" 2>/dev/null | wc -l | tr -d ' ')"
    
    # Top blocks
    echo -e "\n${YELLOW}Top 5 blocks by activity:${NC}"
    sort -k2 -nr "$BLOCK_STATS" | head -5 | while read block count; do
        echo -e "  Block ${BLUE}$block${NC}: $count tickers"
    done
fi