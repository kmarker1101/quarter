# Performance Baseline Measurements

Baseline measurements taken before Phase 7 optimizations.

**Current Status**: We have a working JIT compiler that provides 35x speedup on Fibonacci(30).
Phase 7 will optimize the JIT further by inlining primitives and enabling aggressive LLVM optimizations.

## Test Environment
- Machine: [To be filled]
- Date: 2025-10-24
- Compiler: Rust (debug build)
- Mode: Interpreted (no JIT optimizations)

## Benchmark Results

### Before Phase 7 Optimizations:

| Benchmark | Iterations/Input | Result | Time (JIT) | Time (no JIT) | Speedup |
|-----------|------------------|--------|------------|---------------|---------|
| Fibonacci | n=30 | 1,346,269 | 0.13s | 4.58s | 35x |

### After Phase 7 - Inlined Arithmetic (+, -, *, /):

| Benchmark | Iterations/Input | Result | Time (JIT) | Time (no JIT) | Speedup | Improvement |
|-----------|------------------|--------|------------|---------------|---------|-------------|
| Fibonacci | n=30 | 1,346,269 | **0.10s** | 4.63s | **46x** | **23% faster!** |

**Key Improvements:**
- All arithmetic operations (+, -, *, /) now generate direct LLVM instructions
- No function call overhead for arithmetic
- 23% performance improvement on Fibonacci benchmark
- Speedup increased from 35x to 46x

### After Phase 7 - ALL Primitives Inlined (+, -, *, /, DUP, DROP, SWAP):

| Benchmark | Iterations/Input | Result | Time (JIT) | Time (no JIT) | Speedup | Improvement vs Baseline |
|-----------|------------------|--------|------------|---------------|---------|------------------------|
| Fibonacci | n=30 | 1,346,269 | **0.05s** | 4.67s | **93x** | **2.6x faster (160% improvement)!** |

**Key Improvements:**
- ✅ All arithmetic operations inlined (+, -, *, /)
- ✅ All stack operations inlined (DUP, DROP, SWAP)
- ✅ Zero function call overhead - everything is inline LLVM instructions
- **93x speedup** vs interpreted mode (was 35x at baseline)
- **2.6x faster** than baseline JIT (0.13s → 0.05s)
- **160% performance improvement** overall!

### After Phase 7 - ALL Primitives + Comparisons (<, >, =):

| Benchmark | Iterations/Input | Result | Time (JIT) | Time (no JIT) | Speedup | Improvement vs Baseline |
|-----------|------------------|--------|------------|---------------|---------|------------------------|
| Fibonacci | n=30 | 1,346,269 | **0.04s** | 4.61s | **115x** | **3.25x faster (225% improvement)!** |

**Key Improvements:**
- ✅ All arithmetic operations inlined (+, -, *, /)
- ✅ All stack operations inlined (DUP, DROP, SWAP)
- ✅ All comparison operations inlined (<, >, =)
- ✅ **Zero function call overhead** for all primitives
- **115x speedup** vs interpreted mode (was 35x at baseline)
- **3.25x faster** than baseline JIT (0.13s → 0.04s)
- **225% performance improvement** overall!

### After Phase 7 - ALL 22 Primitives Inlined (COMPLETE):

| Benchmark | Iterations/Input | Result | Time (JIT) | Time (no JIT) | Speedup | Improvement vs Baseline |
|-----------|------------------|--------|------------|---------------|---------|------------------------|
| Fibonacci | n=30 | 1,346,269 | **0.04s** | 4.64s | **116x** | **3.25x faster (225% improvement)!** |

**Final Primitive Set Inlined (22 total):**
- ✅ **Arithmetic (5)**: +, -, *, /, MOD
- ✅ **Stack operations (5)**: DUP, DROP, SWAP, OVER, ROT
- ✅ **Comparisons (6)**: <, >, =, <=, >=, <>
- ✅ **Bitwise operations (4)**: AND, OR, XOR, INVERT
- ✅ **Shift operations (2)**: LSHIFT, RSHIFT

**Achievement Unlocked:**
- 🎯 **116x speedup** vs interpreted mode (was 35x at baseline)
- 🚀 **3.25x faster** than baseline JIT (0.13s → 0.04s)
- ⚡ **Zero function call overhead** - all primitives are direct LLVM instructions
- ✨ All 94 Forth tests passing
- 🏆 **Phase 7 Option 1 COMPLETE!**

## Phase 7 Performance Goals

From issue #30, target improvements with full optimization:

| Benchmark | Current (interpreted) | Target (optimized) | Speedup Goal |
|-----------|----------------------|---------------------|--------------|
| Arithmetic loop (10M) | 5000ms (estimated) | 5ms | 1000x |
| Fibonacci(30) | ~10,000ms | 30ms | ~333x |
| Factorial(12) | ~100ms (estimated) | 1ms | 100x |
| Nested loops (1000x1000) | 1000ms | 10ms | 100x |

## Next Steps

1. Add proper timing infrastructure to measure performance accurately
2. Implement Phase 7 optimizations:
   - Inline primitives (direct LLVM instructions)
   - Stack pointer optimization
   - Enable LLVM optimization passes
   - Constant folding
3. Re-run benchmarks and compare against baseline
4. Scale up iterations once optimizations are in place

## Running Benchmarks

```bash
# Run individual benchmarks
cargo run --release benchmarks/perf_factorial.fth
cargo run --release benchmarks/perf_fibonacci.fth
cargo run --release benchmarks/perf_arithmetic.fth
cargo run --release benchmarks/perf_nested_loops.fth

# With JIT disabled (interpreter only)
cargo run --release -- --no-jit benchmarks/perf_factorial.fth

# With IR dump (to see generated code)
cargo run --release -- --dump-ir benchmarks/perf_factorial.fth
```
