# JIT (Just-In-Time) Compilation

Quarter supports Just-In-Time compilation via LLVM, providing 100-500x performance improvement over interpreted mode while maintaining development flexibility.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [How JIT Works](#how-jit-works)
- [Two-Pass Compilation](#two-pass-compilation)
- [Word Redefinition Handling](#word-redefinition-handling)
- [Performance](#performance)
- [Comparison with Other Modes](#comparison-with-other-modes)
- [Usage Examples](#usage-examples)
- [Troubleshooting](#troubleshooting)
- [Limitations](#limitations)

## Overview

**JIT (Just-In-Time) compilation** converts Forth code to native machine code at runtime using LLVM. Unlike AOT compilation (which produces standalone executables), JIT compilation happens during program execution, combining fast performance with development flexibility.

### Key Features

- **100-500x speedup** over interpreted mode
- **Self-hosting compiler** written in Forth (stdlib/compiler.fth)
- **Batch compilation** of all user-defined words
- **Automatic fallback** to interpreted mode for problematic code
- **No separate build step** required
- **Full recursion support** including RECURSE and tail-call optimization

### When to Use JIT Mode

**Use JIT mode when:**
- Developing Forth programs that need performance
- Testing code before AOT compilation
- Running programs that don't need standalone distribution
- Iterating quickly on performance-critical code

**Use interpreted mode when:**
- Debugging (easier to trace)
- Learning Forth
- Using the REPL interactively
- Code redefines words multiple times in the same file

**Use AOT mode when:**
- Deploying standalone executables
- Maximum optimization needed
- Distributing programs without Quarter interpreter

## Quick Start

```bash
# Run file with JIT compilation
cargo run myprogram.fth --jit

# Run without JIT (interpreted)
cargo run myprogram.fth

# Development workflow
cargo run myprogram.fth --jit    # Fast execution
cargo run myprogram.fth          # Debug issues
```

## How JIT Works

JIT compilation uses Quarter's self-hosting Forth compiler (written in Forth) to generate LLVM IR at runtime:

```
Forth Source Code
    ↓
Parse & Validate
    ↓
Generate AST
    ↓
Pass 1: Define All Words (define_only mode)
    ↓
Forth Compiler (stdlib/compiler.fth)
    ↓
LLVM IR Generation
    ↓
LLVM JIT Engine
    ↓
Pass 2: Execute with Native Code
```

### Self-Hosting Compiler

The Forth compiler that generates LLVM IR is itself written in Forth:

**Location:** `stdlib/compiler.fth` (~3500 lines)

**Capabilities:**
- Parses AST nodes and generates LLVM IR
- Handles all control flow structures (IF/THEN, loops, RECURSE)
- Manages stack operations via LLVM primitives
- Implements tail-call optimization
- Supports string literals (with dual-strategy implementation)

The compiler runs **in interpreted mode** to generate native code for user words, creating a bootstrap chain:
1. Rust interpreter loads compiler.fth
2. Compiler (interpreted) generates LLVM IR for user words
3. LLVM compiles IR to native code
4. User words execute at native speed

## Two-Pass Compilation

JIT mode uses a two-pass approach to handle forward references and enable batch compilation:

### Pass 1: Define Only (define_only=true)

```forth
: WORD1 42 ;        # Parsed, AST stored, not executed
WORD1 .             # Skipped (define_only mode)
: WORD2 WORD1 + ;   # Parsed, WORD1 is now known
```

**Actions:**
- Parse all word definitions
- Build AST for each word
- Store in dictionary as `Word::Compiled(ast)`
- **Skip all execution**
- Track word definitions for redefinition detection

**Purpose:**
- Resolve forward references
- Collect all words before compilation
- Detect same-file redefinitions

### Batch Compilation

After Pass 1, all words are compiled together:

```rust
// Pseudocode
for each word in dictionary:
    if word is Compiled(ast):
        generate_llvm_ir(ast)
        compile_to_native()
        store as Word::JITCompiled(fn_ptr)
        freeze_word(name)  // Prevent redefinition
```

**Result:** All user words are now native code

### Pass 2: Execute (define_only=false)

```forth
: WORD1 42 ;        # Skipped (frozen, already compiled)
WORD1 .             # Executes native code → prints 42
: WORD2 WORD1 + ;   # Skipped (frozen)
```

**Actions:**
- Skip word definitions (already compiled and frozen)
- Execute all other code (prints, calculations, etc.)
- Use native code for compiled words
- Use interpreter for immediate execution

## Word Redefinition Handling

### The Problem

Before the fix (Issue #68), JIT mode incorrectly handled same-file word redefinitions:

```forth
: WORD1 42 ;
WORD1 .          \ Should print 42
: WORD1 99 ;
WORD1 .          \ Should print 99

\ BUG: Printed "99 99" instead of "42 99"
```

**Why it happened:**
- Pass 1: Only final definition (99) survived
- Batch compile: WORD1 compiled as `99`
- Pass 2: Both executions used same compiled version

### The Solution

JIT mode now **detects same-file redefinitions** and **automatically falls back to interpreted mode**:

```forth
: WORD1 42 ;
WORD1 .          \ Interpreted: prints 42
: WORD1 99 ;
WORD1 .          \ Interpreted: prints 99

\ Output: "42 99" ✓
\ Warning: "Word redefinition detected in 'file.fth'"
\ Info: "Falling back to interpreted mode for this file."
```

### Detection Logic

**Same-file redefinition (detected):**
```forth
\ file1.fth
: WORD1 42 ;
: WORD1 99 ;     # DETECTED: Same word, same file
                 # → Fallback to interpreted mode
```

**Cross-file redefinition (NOT detected, works normally):**
```forth
\ stdlib/core.fth
: 2DUP OVER OVER ;

\ myfile.fth (JIT mode)
: 2DUP OVER OVER ;    # OK: Different file, uses JIT normally
```

### Implementation Details

**Dictionary tracking:**
- `current_file_words: HashSet<String>` - Words defined in current file
- `has_redefinitions: bool` - Flag for detection

**Detection flow:**
1. `start_file_tracking()` - Called at file load (Pass 1)
2. `track_word_definition(name)` - Called for each `: WORD ... ;`
3. If word already in `current_file_words` → set `has_redefinitions = true`
4. After Pass 1: Check `has_redefinitions()`
5. If true: Skip batch compilation, execute in interpreted mode

**Code locations:**
- Dictionary: `src/dictionary.rs` lines 273-302
- Detection: `src/lib.rs` lines 1146-1148, 1242-1245
- Fallback: `src/main.rs` lines 709-742

## Performance

### Benchmark Results

**Test: Factorial(12) computed 10,000 times**

| Mode | Time | Speedup | Notes |
|------|------|---------|-------|
| Interpreted | 5000ms | 1x | Baseline |
| JIT (no redefs) | 50ms | 100x | Native code |
| JIT (with redefs) | 5000ms | 1x | Falls back to interpreted |
| AOT (-O2) | 45ms | 111x | Slightly faster than JIT |

### Performance Characteristics

**Startup Time:**
- **Interpreter**: Instant (~0ms overhead)
- **JIT**: ~100-300ms (loading + compiling stdlib/compiler.fth + batch compile)
- **AOT**: Instant (pre-compiled)

**Execution Speed:**
- **Primitives**: Same as interpreted (native Rust functions)
- **User words**: 100-500x faster (native code vs AST traversal)
- **Recursion**: Same native speed as AOT
- **TCO**: Verified up to 500,000 recursive calls

**Memory:**
- **Additional overhead**: ~10-20MB for LLVM JIT engine
- **Code cache**: Minimal (~few KB per compiled word)

### When JIT is Slower

JIT may be slower than interpreted for:
- **Very short programs** (~5-10 lines): Compilation overhead dominates
- **Programs with redefinitions**: Automatic fallback to interpreted mode
- **REPL single-line execution**: No batch compilation benefit

## Comparison with Other Modes

| Feature | Interpreted | JIT | AOT |
|---------|------------|-----|-----|
| **Speed** | 1x | 100-500x | 100-500x |
| **Startup** | Instant | ~200ms | Instant |
| **Build step** | None | None | Required |
| **Distribution** | Needs Quarter | Needs Quarter | Standalone |
| **REPL** | ✅ Yes | ✅ Yes | ❌ No |
| **Redefinitions** | ✅ Always works | ⚠️ Auto-fallback | ✅ Works |
| **Debugging** | ✅ Easy | ⚠️ Harder | ⚠️ Harder |
| **TCO** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Recursion** | ✅ Yes | ✅ Yes | ✅ Yes |

## Usage Examples

### Example 1: Simple Program

**Source (fibonacci.fth):**
```forth
: FIB ( n -- fib(n) )
  DUP 2 < IF
  ELSE
    DUP 1 - RECURSE
    SWAP 2 - RECURSE
    +
  THEN ;

: MAIN
  10 FIB . CR
;

MAIN
```

**Run:**
```bash
$ cargo run fibonacci.fth --jit
55
```

**Performance:**
- Interpreted: ~500ms
- JIT: ~5ms (100x faster)
- AOT: ~4ms (125x faster)

### Example 2: Redefinition Example

**Source (redefine.fth):**
```forth
: VERSION 1 ;
." Version: " VERSION . CR

: VERSION 2 ;
." Version: " VERSION . CR
```

**Run with JIT:**
```bash
$ cargo run redefine.fth --jit
Warning: Word redefinition detected in 'redefine.fth'
Falling back to interpreted mode for this file.
Version: 1
Version: 2
```

**Behavior:**
- JIT detects redefinition of `VERSION`
- Falls back to interpreted mode
- Produces correct output

### Example 3: Performance Test

**Source (benchmark.fth):**
```forth
: FACTORIAL ( n -- n! )
  DUP 1 <= IF
    DROP 1
  ELSE
    DUP 1 - RECURSE *
  THEN ;

: BENCH
  10000 0 DO
    12 FACTORIAL DROP
  LOOP ;

BENCH
```

**Compare modes:**
```bash
# Interpreted: ~5 seconds
$ time cargo run benchmark.fth
real    0m5.234s

# JIT: ~50 milliseconds
$ time cargo run benchmark.fth --jit
real    0m0.257s  # (includes ~200ms JIT startup)
```

### Example 4: Development Workflow

```bash
# 1. Develop and test with interpreter (fast iteration)
$ cargo run myapp.fth
# ... fix bugs ...

# 2. Test performance with JIT
$ cargo run myapp.fth --jit
# ... optimize hot paths ...

# 3. Deploy with AOT
$ cargo run -- --compile myapp.fth -o myapp -O3
$ ./myapp
```

## Troubleshooting

### Warning: Word Redefinition Detected

**Problem:**
```
Warning: Word redefinition detected in 'myfile.fth'
Falling back to interpreted mode for this file.
```

**Cause:**
File redefines a word multiple times:
```forth
: WORD1 42 ;
: WORD1 99 ;     # Redefinition in same file
```

**Solutions:**

1. **Accept interpreted mode** (simplest)
   - Warning is informational
   - Code still works correctly
   - Just slower for this file

2. **Split into separate files** (keeps JIT)
```forth
\ file1.fth
: WORD1 42 ;

\ file2.fth
: WORD1 99 ;     # OK: Different file, both use JIT
```

3. **Use different word names**
```forth
: WORD1-V1 42 ;
: WORD1-V2 99 ;
```

### JIT Compilation Failed

**Problem:**
```
Batch compilation failed: <error message>
```

**Common causes:**
- Missing LLVM 18.x installation
- Corrupted compiler.fth
- Out of memory (very large programs)

**Solutions:**
```bash
# Check LLVM installation
llvm-config --version    # Should show 18.x

# Reinstall LLVM
brew install llvm@18     # macOS
sudo apt install llvm-18 # Ubuntu

# Run without JIT
cargo run myfile.fth
```

### Slow JIT Startup

**Problem:**
JIT mode takes several seconds to start

**Cause:**
First run compiles stdlib + compiler (large)

**Solutions:**
- **Accept delay** - Only happens once per execution
- **Use interpreted mode** for quick tests
- **Use AOT** for production deployment

### Segfault or Crash

**Problem:**
Program crashes with segmentation fault

**Causes:**
- Stack overflow (non-tail recursion)
- Memory corruption bug
- LLVM codegen issue

**Debug steps:**
```bash
# 1. Try interpreted mode (isolates JIT issues)
cargo run myfile.fth

# 2. Enable debug output
QUARTER_DEBUG=1 cargo run myfile.fth --jit

# 3. Use LLDB/GDB
lldb cargo run -- myfile.fth --jit

# 4. Report bug with minimal reproduction
```

## Limitations

### Current Limitations

1. **No REPL JIT Compilation**
   - REPL uses interpreted mode only
   - JIT only works with file execution
   - Reason: Two-pass compilation requires complete program

2. **Same-File Redefinitions**
   - Automatically falls back to interpreted mode
   - Cross-file redefinitions work normally
   - Impact: Slower execution for files with redefinitions

3. **Compilation Overhead**
   - ~200ms startup time for JIT
   - Not suitable for very short programs
   - Interpreted mode faster for <10 lines

4. **No Incremental Compilation**
   - All words recompiled each run
   - Cannot cache compiled code between runs
   - Future: Could add compilation cache

5. **Memory Overhead**
   - LLVM JIT engine: ~10-20MB RAM
   - Usually not a problem on modern systems

### Known Issues

- **Issue #68**: ~~JIT mode fails with word redefinition~~ **FIXED** (auto-fallback implemented)
- String output (`."`) works via dual-strategy implementation (see [LLVM Global Strings](llvm-global-strings-notes.md))

### Planned Improvements

- [ ] REPL JIT compilation support
- [ ] Compilation result caching
- [ ] Incremental compilation
- [ ] Better error messages for JIT failures
- [ ] Profiling support for JIT code

## See Also

- **[AOT Compilation](aot-compilation.md)** - Standalone executables
- **[LLVM Global Strings](llvm-global-strings-notes.md)** - Dual-strategy string implementation
- **[Adding LLVM Primitives](adding-llvm-primitives.md)** - Extending the compiler
- **[Control Flow](control-flow.md)** - RECURSE and tail-call optimization
- **[Main README](../README.md)** - Project overview and quick start
