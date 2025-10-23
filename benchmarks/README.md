# Quarter Forth Benchmarks

This directory contains benchmark programs to test JIT compilation performance.

## Benchmark Files

### simple_arithmetic.fth
Tests JIT-compilable words (arithmetic operations only):
- `ADDCHAIN` - Addition chain
- `SQUARE` - Squares a number (DUP *)
- `DOUBLE` - Doubles a number (DUP +)
- `COMPLEX` - Complex arithmetic expression
- `DIVCHAIN` - Division chain

**Expected**: These words WILL be JIT compiled and should show performance improvement.

### control_flow.fth
Tests words with control flow (NOT currently JIT-compilable):
- `LOOP-ADD` - Loop with addition
- `LOOP-SQUARE` - Loop with squaring
- `COUNTDOWN` - Countdown using BEGIN/UNTIL
- `FACTORIAL` - Recursive factorial

**Expected**: These words will NOT be JIT compiled (fall back to interpreter) due to unsupported control flow constructs.

## Running Benchmarks

### Test JIT-compiled arithmetic:
```bash
cargo run --release benchmarks/simple_arithmetic.fth
```

### Test with JIT disabled:
```bash
cargo run --release -- --no-jit benchmarks/simple_arithmetic.fth
```

### Test control flow (interpreter-only):
```bash
cargo run --release benchmarks/control_flow.fth
```

## Current JIT Support

The JIT compiler currently supports:
- ✅ PushNumber (literal numbers)
- ✅ CallWord for: `*`, `DUP`, `DROP`, `SWAP`, `+`, `-`, `/`
- ✅ Sequence (multiple operations)

NOT supported (falls back to interpreter):
- ❌ Control flow (IF/THEN/ELSE, DO/LOOP, BEGIN/UNTIL, etc.)
- ❌ Calling user-defined words
- ❌ String operations
- ❌ Memory operations (@, !, etc.)
- ❌ Return stack operations (>R, R>, etc.)

## Performance Expectations

For JIT-compiled words (simple arithmetic):
- Expected 10-100x speedup for tight computational loops
- Cold start (JIT compile time): < 10ms per word

For interpreter-only words (control flow):
- No performance difference between JIT and --no-jit modes
- Both use interpreter execution
