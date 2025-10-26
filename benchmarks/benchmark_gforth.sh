#!/bin/bash
# Benchmark Quarter vs Gforth
# Compares performance on standard Forth benchmarks

set -e

echo "========================================"
echo "Quarter vs Gforth Performance Benchmark"
echo "========================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

QUARTER_BIN="./target/debug/quarter"

# Check if binaries exist
if [ ! -f "$QUARTER_BIN" ]; then
    echo "Building Quarter..."
    cargo build --quiet
fi

if ! command -v gforth &> /dev/null; then
    echo "Error: gforth not found. Install with: brew install gforth"
    exit 1
fi

echo "Environment:"
echo "  Quarter: $QUARTER_BIN"
echo "  Gforth:  $(which gforth) ($(gforth --version 2>&1 | head -1))"
echo ""

# Function to time a command
time_command() {
    local cmd="$1"
    local label="$2"

    echo -n "  ${label}: "

    # Use time command and capture real time
    local start=$(date +%s%N)
    eval "$cmd" > /dev/null 2>&1
    local end=$(date +%s%N)

    local elapsed=$(( (end - start) / 1000000 ))  # Convert to milliseconds
    echo "${elapsed}ms"

    echo "$elapsed"
}

# Function to run benchmark
run_benchmark() {
    local bench_file="$1"
    local bench_name="$2"

    echo -e "${BLUE}${bench_name}${NC}"
    echo "  File: $bench_file"

    # Quarter interpreted
    local q_interp=$(time_command "$QUARTER_BIN $bench_file" "Quarter (interpreted)")

    # Quarter JIT
    local q_jit=$(time_command "$QUARTER_BIN $bench_file --jit" "Quarter (JIT)")

    # Gforth
    local gforth_time=$(time_command "gforth $bench_file -e bye" "Gforth")

    # Calculate speedups
    echo "  ---"
    if [ "$q_jit" -gt 0 ]; then
        local jit_vs_interp=$(echo "scale=2; $q_interp / $q_jit" | bc)
        echo -e "  ${GREEN}Quarter JIT speedup vs interpreted: ${jit_vs_interp}x${NC}"
    fi

    if [ "$gforth_time" -gt 0 ] && [ "$q_jit" -gt 0 ]; then
        local vs_gforth=$(echo "scale=2; $gforth_time / $q_jit" | bc)
        if [ $(echo "$vs_gforth > 1" | bc) -eq 1 ]; then
            echo -e "  ${GREEN}Quarter JIT vs Gforth: ${vs_gforth}x faster${NC}"
        else
            local gforth_faster=$(echo "scale=2; $q_jit / $gforth_time" | bc)
            echo -e "  ${YELLOW}Gforth vs Quarter JIT: ${gforth_faster}x faster${NC}"
        fi
    fi

    if [ "$gforth_time" -gt 0 ] && [ "$q_interp" -gt 0 ]; then
        local interp_vs_gforth=$(echo "scale=2; $gforth_time / $q_interp" | bc)
        if [ $(echo "$interp_vs_gforth > 1" | bc) -eq 1 ]; then
            echo -e "  Quarter interpreted vs Gforth: ${interp_vs_gforth}x faster"
        else
            local gforth_faster=$(echo "scale=2; $q_interp / $gforth_time" | bc)
            echo -e "  Gforth vs Quarter interpreted: ${gforth_faster}x faster"
        fi
    fi

    echo ""
}

# Run benchmarks
echo "========================================"
echo "Running Benchmarks..."
echo "========================================"
echo ""

run_benchmark "benchmarks/perf_factorial.fth" "Factorial(12)"
run_benchmark "benchmarks/perf_fibonacci.fth" "Fibonacci(30)"
run_benchmark "benchmarks/perf_arithmetic.fth" "Arithmetic Loop (100K iterations)"
run_benchmark "benchmarks/perf_nested_loops.fth" "Nested Loops (1000x1000)"

echo "========================================"
echo "Benchmark Complete!"
echo "========================================"
