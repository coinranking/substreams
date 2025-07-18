#!/bin/bash

# Script to verify 24-hour rolling volume behavior

echo "Testing 24-hour rolling volume implementation..."
echo "================================================"

# Test 1: Initial volume accumulation (first hour)
echo -e "\nTest 1: First hour of data (blocks 12369621-12370221)"
./test.sh -s 12369621 -e 12370221 -f 2>&1 | grep -A 10 "rollingVolumes" | grep -E "poolAddress|token0Volume24h|token1Volume24h" | head -15

# Test 2: Multiple hours accumulation  
echo -e "\nTest 2: Four hours of data (blocks 12369621-12371221)"
./test.sh -s 12369621 -e 12371221 -f 2>&1 | tail -50 | grep -A 10 "rollingVolumes" | grep -E "poolAddress|token0Volume24h|token1Volume24h" | head -15

echo -e "\nNote: Volumes should accumulate over time as more 5-minute periods are processed."
echo "Each 5-minute period = ~25 blocks on Ethereum mainnet"
echo "24 hours = 288 periods = ~7,200 blocks"