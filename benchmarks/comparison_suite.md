# Quarter vs Gforth Performance Comparison

## Summary

This document outlines a comprehensive performance comparison between Quarter Forth and Gforth, focusing on both interpreted and JIT-compiled execution.

## Why This Matters

**Gforth** is a mature, highly optimized Forth system that:
- Has been developed for 30+ years
- Uses threaded code interpretation
- Is the standard reference for Forth performance
- Represents "production-quality" Forth

**Quarter** aims to:
- Match or exceed Gforth performance with JIT compilation
- Provide better performance than Gforth in computational workloads
- Learn where optimizations are most needed

## Current Benchmark Status

### Existing Benchmarks (in `benchmarks/`)

1. **perf_factorial.fth** - Factorial(12) with recursion
2. **perf_fibonacci.fth** - Fibonacci(30) with double recursion
3. **perf_arithmetic.fth** - Arithmetic loop (100K iterations)
4. **perf_nested_loops.fth** - Nested DO/LOOP (1000x1000)

### Issues with Gforth Compatibility

**Problem**: The existing benchmarks use direct recursion (word calling itself by name), which Gforth doesn't allow during compilation.

**Example (doesn't work in Gforth)**:
```forth
: FACTORIAL ( n -- n! )
    DUP 1 <= IF
        DROP 1
    ELSE
        DUP 1 - FACTORIAL *    \ Can't reference FACTORIAL here!
    THEN ;
```

**Solution**: Use `RECURSE` for recursion (standard Forth word):
```forth
: FACTORIAL ( n -- n! )
    DUP 1 <= IF
        DROP 1
    ELSE
        DUP 1 - RECURSE *      \ Use RECURSE instead
    THEN ;
```

### Quarter's Current Advantage

Quarter supports both:
- Direct recursion: `: FACTORIAL ... FACTORIAL ... ;`
- Standard `RECURSE` (should add this!)

Gforth only supports `RECURSE` (the standard way).

## Recommended Approach

### Option 1: Add RECURSE to Quarter (Recommended)

**Pros**:
- Standard Forth compliance
- Enables direct comparison with Gforth
- Better for portability
- Educational value

**Cons**:
- Small amount of work to implement

**Implementation**:
```rust
// In src/words.rs or during parsing
// RECURSE should compile to a self-reference in the current word being defined
```

**Estimated work**: 2-4 hours

### Option 2: Create Dual Benchmark Files

Create two versions of each benchmark:
- `perf_factorial.fth` - Quarter version (direct recursion)
- `bench_factorial_portable.fth` - Portable version (with RECURSE)

**Pros**:
- No changes to Quarter needed
- Can compare immediately

**Cons**:
- Maintaining two versions of each benchmark
- Not following Forth standards

### Option 3: Use Non-Recursive Benchmarks

Focus on benchmarks that don't need recursion:
- Arithmetic loops
- Nested loops
- Iterative algorithms

**Pros**:
- Works immediately with both systems
- Tests different aspects of performance

**Cons**:
- Doesn't test function call overhead
- Misses important use cases

## Proposed Benchmark Suite

### 1. Arithmetic Performance

**Test**: Pure arithmetic operations
```forth
: BENCH-ARITHMETIC ( -- )
    0
    1000000 0 DO
        I 2 * 3 + 5 - 7 * 11 / +
    LOOP
    DROP ;
```

**What it tests**: Inline arithmetic, stack operations, DO/LOOP

**Expected**: Quarter JIT should be 10-100x faster than interpreted, competitive with Gforth

### 2. Factorial (Iterative)

**Test**: Non-recursive factorial
```forth
: FACTORIAL ( n -- n! )
    1 SWAP
    1 + 1 DO
        I *
    LOOP ;
```

**What it tests**: DO/LOOP, multiplication

**Expected**: Quarter JIT competitive with Gforth

### 3. Fibonacci (Iterative)

**Test**: Non-recursive Fibonacci
```forth
: FIBONACCI ( n -- fib[n] )
    0 1 ROT
    0 DO
        OVER + SWAP
    LOOP
    DROP ;
```

**What it tests**: DO/LOOP, stack manipulation

**Expected**: Quarter JIT competitive with Gforth

### 4. Sieve of Eratosthenes

**Test**: Prime number sieve
```forth
VARIABLE SIEVE 10000 ALLOT

: PRIME-SIEVE ( n -- count )
    SIEVE OVER 0 FILL
    0 SWAP                    \ counter limit
    2 DO
        SIEVE I + C@ 0= IF
            1 +               \ Increment counter
            I DUP * SIEVE + SWAP DO
                SIEVE I + 1 SWAP C!
            I +LOOP
        THEN
    LOOP ;
```

**What it tests**: Memory operations, nested loops, complex logic

**Expected**: Tests real-world algorithm performance

### 5. Nested Loop Intensive

**Test**: Nested loop with computation
```forth
: NESTED-COMPUTE ( -- sum )
    0
    1000 0 DO
        1000 0 DO
            I J + DUP * +
        LOOP
    LOOP ;
```

**What it tests**: Nested loops, I/J access, arithmetic

**Expected**: Quarter JIT significantly faster than interpreted

### 6. Memory Operations

**Test**: Array manipulation
```forth
VARIABLE ARRAY 10000 CELLS ALLOT

: ARRAY-SUM ( -- sum )
    \ Fill array
    10000 0 DO
        I CELLS ARRAY + I SWAP !
    LOOP
    \ Sum array
    0
    10000 0 DO
        I CELLS ARRAY + @ +
    LOOP ;
```

**What it tests**: Memory access patterns, address arithmetic

**Expected**: Shows memory operation overhead

## Measurement Methodology

### Timing Approach

Use `/usr/bin/time -p` for accurate measurements:

```bash
# Quarter interpreted
/usr/bin/time -p ./quarter benchmark.fth 2>&1 | grep real

# Quarter JIT
/usr/bin/time -p ./quarter benchmark.fth --jit 2>&1 | grep real

# Gforth
/usr/bin/time -p gforth benchmark.fth -e bye 2>&1 | grep real
```

### What to Measure

For each benchmark:
1. **Quarter Interpreted** - Baseline AST execution
2. **Quarter JIT** - LLVM-compiled execution
3. **Gforth** - Standard Forth interpreter

### Metrics

- **Raw time** (milliseconds)
- **Quarter JIT speedup** vs interpreted
- **Quarter JIT speedup** vs Gforth (if positive)
- **Gforth speedup** vs Quarter JIT (if Gforth is faster)

### Example Results Table

| Benchmark | Quarter Interpreted | Quarter JIT | Gforth | JIT Speedup | vs Gforth |
|-----------|---------------------|-------------|--------|-------------|-----------|
| Arithmetic (1M) | 2500ms | 25ms | 150ms | 100x | 6x faster |
| Factorial(12) | 45ms | 2ms | 8ms | 22.5x | 4x slower |
| Nested loops | 3200ms | 35ms | 180ms | 91x | 5x faster |

## Implementation Priority

1. **✅ High Priority**: Add RECURSE support to Quarter
   - Enables standard Forth compatibility
   - Required for recursive benchmarks
   - Small implementation effort

2. **✅ High Priority**: Create non-recursive benchmark suite
   - Works immediately with both systems
   - Tests important use cases
   - Shows real-world performance

3. **Medium Priority**: Create benchmark script
   - Automates testing both systems
   - Generates comparison tables
   - Tracks performance over time

4. **Low Priority**: Add recursive benchmarks
   - Only after RECURSE is implemented
   - Nice to have, but iterative versions work

## Next Steps

1. Implement `RECURSE` in Quarter (~2-4 hours)
2. Create portable benchmark files using RECURSE
3. Create comprehensive benchmark script
4. Run full comparison and document results
5. Identify optimization opportunities based on results

## Expected Outcomes

Based on Phase 7 results (116x speedup on Fibonacci):

**Quarter JIT should be**:
- 10-100x faster than Quarter interpreted ✅
- Competitive with or faster than Gforth on computational tasks ✅
- Possibly slower than Gforth on I/O and memory-heavy tasks ❓

**Key insights we'll gain**:
- Where JIT compilation helps most
- Where Gforth's optimizations are superior
- What optimizations Quarter needs most
- Real-world performance vs micro-benchmarks
