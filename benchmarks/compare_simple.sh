#!/bin/bash
# Simple performance comparison: Quarter vs Gforth

BENCH="benchmarks/bench_arithmetic_portable.fth"

echo "========================================="
echo "Performance Comparison: Arithmetic Benchmark"
echo "100,000 iterations of: I 2 * 3 + 4 - 5 /"
echo "========================================="
echo ""

echo "Quarter (interpreted):"
time ./target/debug/quarter $BENCH

echo ""
echo "Quarter (JIT):"
time ./target/debug/quarter $BENCH --jit

echo ""
echo "Gforth:"
time gforth $BENCH -e bye

echo ""
echo "========================================="
echo "Done!"
echo "========================================="
