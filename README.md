# Quarter - A Self-Hosting Forth Interpreter

Quarter is a Forth interpreter written in Rust with a **self-hosting JIT compiler** that compiles Forth code to native machine code via LLVM.

[![Tests](https://img.shields.io/badge/tests-139%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

## Status: Self-Hosting Achieved ✅

Quarter has achieved **true self-compilation**: The Forth compiler is written in Forth and successfully compiles itself to native code. This represents a complete bootstrap cycle where the compiler can compile its own source code.

## Quick Start

```bash
# Build the project
cargo build

# Interactive REPL (interpreted mode)
cargo run

# Run a Forth file (interpreted)
cargo run examples/factorial.fth

# Run with JIT compilation (native code)
cargo run examples/factorial.fth --jit

# Run all tests
cargo test
```

## Key Features

### ✨ Highlights

- **Self-hosting compiler** written entirely in Forth
- **Two execution modes**: Interpreted (AST evaluation) and JIT (native code via LLVM)
- **Full recursion support**: Both `RECURSE` word and direct recursion
- **Comprehensive test suite**: 139 tests across unit and integration testing
- **Interactive REPL** with history and line editing
- **Standard Forth compliance**: Case-insensitive, standard control flow, memory model

### Execution Modes

#### 1. Interpreted Mode (Default)
Direct AST evaluation, immediate execution, no compilation overhead.

```bash
cargo run                           # REPL
cargo run myprogram.fth             # Run file
```

#### 2. JIT Mode
Batch compiles all user-defined words to native code via LLVM for 100-500x speedup.
Uses the self-hosting Forth compiler to generate LLVM IR.

```bash
cargo run myprogram.fth --jit       # JIT compile and run
```

## Command Line Arguments

```bash
quarter [OPTIONS] [FILE]

Arguments:
  [FILE]              Forth source file to execute (.fth, .forth, .qtr, .quarter)

Options:
  --jit               Enable JIT compilation (batch compiles all words to native code)
  --compile-stdlib    Compile standard library to native code
  --no-jit            Disable JIT compilation (keep interpreted)
  --dump-ir           Dump LLVM IR for debugging
  --verify-ir         Verify LLVM IR correctness

Examples:
  quarter                                  # Interactive REPL (interpreted)
  quarter script.fth                       # Run script (interpreted)
  quarter script.fth --jit                 # Run script (JIT compiled)
  cargo run                                # REPL (interpreted)
  cargo run script.fth --jit               # JIT mode
  cargo run tests/recurse_tests.fth --jit  # Run tests in JIT mode
```

## Building and Running

### Basic Commands

```bash
# Build (debug mode)
cargo build

# Build (release mode, optimized)
cargo build --release

# Run interactive REPL
cargo run

# Run a Forth source file
cargo run myprogram.fth

# Run with JIT compilation
cargo run myprogram.fth --jit

# Run release build (JIT mode)
cargo run --release myprogram.fth --jit
```

### Running Tests

```bash
# Run all tests (139 total)
cargo test

# Run specific test
cargo test test_factorial

# Run with output
cargo test -- --nocapture

# Run only integration tests
cargo test --test forth_integration_tests

# Run specific integration test
cargo test test_recurse_jit
cargo test test_tco_interpreted
```

**Test Suite** (139 tests):
- 133 unit tests (Rust)
- 6 integration tests (Forth programs in both interpreted and JIT modes)
  - `test_forth_tests_interpreted` - Main test suite (interpreted)
  - `test_forth_tests_jit` - Main test suite (JIT)
  - `test_tco_interpreted` - Tail call optimization tests
  - `test_tco_jit` - TCO in JIT mode
  - `test_recurse_interpreted` - RECURSE word tests
  - `test_recurse_jit` - RECURSE in JIT mode

## Implemented Features

### Core Forth Words

**Arithmetic**: `+`, `-`, `*`, `/`, `/MOD`, `MOD`, `NEGATE`, `ABS`, `MIN`, `MAX`, `1+`, `1-`, `2+` ... `11+`, `2*`, `2/`

**Comparison**: `<`, `>`, `=`, `<>`, `<=`, `>=`, `0=`, `0<`, `0>`

**Stack Manipulation**: `DUP`, `DROP`, `SWAP`, `OVER`, `ROT`, `-ROT`, `PICK`, `NIP`, `TUCK`, `2DUP`, `2DROP`, `2SWAP`, `2OVER`, `DEPTH`

**Return Stack**: `>R`, `R>`, `R@`

**Memory Access**: `!`, `@`, `C!`, `C@`, `+!`

**Memory Allocation**: `HERE`, `ALLOT`, `,`, `CELLS`, `CELL+`, `VARIABLE`, `CONSTANT`, `CREATE`

**Stack Pointers**: `SP@`, `SP!`, `RP@`, `RP!`

**Bitwise Logic**: `AND`, `OR`, `XOR`, `INVERT`, `LSHIFT`, `RSHIFT`

**I/O**: `.`, `U.`, `.R`, `U.R`, `.S`, `CR`, `EMIT`, `KEY`, `SPACE`, `SPACES`, `."` (string output)

**String Literals**: `S"` (push address and length to stack)

**Control Flow**:
- `IF/THEN/ELSE` - Conditional execution
- `BEGIN/UNTIL` - Post-test loop
- `BEGIN/WHILE/REPEAT` - Pre-test loop
- `DO/LOOP` - Counted loop
- `+LOOP` - Variable increment loop
- `I`, `J` - Loop indices
- `LEAVE` - Early loop exit
- `EXIT` - Early word return
- `RECURSE` - Self-recursion (works in both interpreted and JIT modes!)

**Constants**: `TRUE` (-1), `FALSE` (0), `BL` (32 - space character)

**Word Definition**: `:` and `;` - Define new words

**File Loading**: `INCLUDE`, `INCLUDED` - Load and execute Forth files

**Comments**: `\` (line comment), `( )` (inline comment)

### LLVM Primitives (for Self-Hosting Compiler)

The Forth compiler has access to 50+ LLVM primitives for code generation:

**Context/Module**: `LLVM-CREATE-CONTEXT`, `LLVM-CREATE-MODULE`, `LLVM-DECLARE-EXTERNAL`, `LLVM-DUMP-MODULE`

**Functions**: `LLVM-CREATE-FUNCTION`, `LLVM-MODULE-GET-FUNCTION`, `LLVM-GET-PARAM`

**Basic Blocks**: `LLVM-CREATE-BLOCK`, `LLVM-POSITION-AT-END`, `LLVM-GET-INSERT-BLOCK`

**IR Generation**: `LLVM-BUILD-CONST-INT`, `LLVM-BUILD-ADD`, `LLVM-BUILD-SUB`, `LLVM-BUILD-MUL`, `LLVM-BUILD-LOAD`, `LLVM-BUILD-STORE`, `LLVM-BUILD-GEP`

**Control Flow**: `LLVM-BUILD-BR`, `LLVM-BUILD-COND-BR`, `LLVM-BUILD-PHI`, `LLVM-PHI-ADD-INCOMING`

**Comparison**: `LLVM-BUILD-ICMP`

**Function Calls**: `LLVM-BUILD-CALL`

**Returns**: `LLVM-BUILD-RET-VOID`, `LLVM-BUILD-RET`

**JIT**: `LLVM-CREATE-JIT`, `LLVM-GET-FUNCTION`, `REGISTER-JIT-WORD`

**AST Inspection**: `AST-TYPE`, `AST-GET-NUMBER`, `AST-GET-WORD`, `AST-SEQ-LENGTH`, `AST-SEQ-CHILD`, `AST-IF-THEN`, `AST-IF-ELSE`, `AST-LOOP-BODY`

## Architecture Overview

### Project Structure

```
quarter/
├── src/
│   ├── main.rs              # REPL, CLI argument parsing
│   ├── lib.rs               # Parser, file loading
│   ├── stack.rs             # Data stack (64-bit cells in memory)
│   ├── dictionary.rs        # Word dictionary (HashMap)
│   ├── words.rs             # Built-in primitive words
│   ├── ast.rs               # Abstract Syntax Tree
│   ├── llvm_codegen.rs      # Rust-based LLVM code generator
│   ├── llvm_forth.rs        # LLVM primitives for Forth compiler
│   └── ast_forth.rs         # AST inspection for Forth compiler
│
├── forth/
│   └── compiler.fth         # Self-hosting Forth compiler (666 lines)
│
├── stdlib/
│   ├── core.fth             # Core Forth standard library
│   └── test-framework.fth   # Unit test framework (T{ -> }T)
│
├── tests/
│   ├── *.rs                 # Rust unit tests (133 tests)
│   ├── run-all-tests.fth    # Main Forth test suite
│   ├── tco_tests.fth        # Tail call optimization tests
│   ├── recurse_tests.fth    # RECURSE word tests
│   └── forth_integration_tests.rs  # Integration tests (6 tests)
│
└── benchmarks/
    ├── perf_factorial.fth   # Factorial benchmark
    ├── perf_fibonacci.fth   # Fibonacci benchmark
    └── *.fth                # Other benchmarks
```

### Memory Layout

8MB byte-addressable memory space with 8-byte (64-bit) cells:

```
0x000000-0x00FFFF  Data Stack      (64KB, 8K cells, grows upward)
0x010000-0x01FFFF  Return Stack    (64KB, 8K cells, grows upward)
0x020000-0x7FFFFF  User Memory     (~7.5MB for HERE/ALLOT/VARIABLE)

Compiler Scratch:
  300,000-301,999  Temporary buffers
  302,000+         Word name storage

LLVM Objects:
  400,000+         LLVM handle storage
```

### Compilation Flow

Example: Compiling `: SQUARE DUP * ;`

1. **Parse** → AST: `Sequence[CallWord("DUP"), CallWord("*")]`

2. **Validate** → Check all referenced words exist

3. **Choose Compiler:**
   - Default: Store as interpreted `Word::Compiled(ast)`
   - `--jit`: Self-hosting Forth compiler (stdlib/compiler.fth) generates LLVM IR and batch compiles all words

4. **JIT Compilation** (if enabled):
   - Creates LLVM module and function
   - Walks AST, emitting IR for each node
   - For `DUP`: `call void @quarter_dup(ptr %memory, ptr %sp, ptr %rp)`
   - For `*`: `call void @quarter_mul(ptr %memory, ptr %sp, ptr %rp)`
   - Creates JIT execution engine
   - Returns native function pointer

5. **Register** → Store as `Word::JITCompiled(fn_ptr)` or `Word::Compiled(ast)`

6. **Execute** → JIT calls native code directly, interpreted evaluates AST

## Example Usage

### Interactive REPL

```forth
$ cargo run
Forth Interpreter v0.2
Type 'quit' to exit

quarter> 5 3 + .
8 ok

quarter> : SQUARE DUP * ;
ok

quarter> 7 SQUARE .
49 ok

quarter> : FACTORIAL ( n -- n! )
compiled   DUP 1 <= IF
compiled     DROP 1
compiled   ELSE
compiled     DUP 1 - RECURSE *
compiled   THEN ;
ok

quarter> 5 FACTORIAL .
120 ok

quarter> .S
<0> ok

quarter> quit
```

### Running Forth Files

```bash
# Simple program
$ cat hello.fth
." Hello, Forth!" CR
5 FACTORIAL .

$ cargo run hello.fth
Forth Interpreter v0.2
Loading hello.fth
Hello, Forth!
120

# With JIT compilation
$ cargo run hello.fth --jit
Forth Interpreter v0.2
Loading hello.fth
120
```

### Recursion Examples

```forth
# Non-tail recursion with RECURSE
: FACTORIAL ( n -- n! )
  DUP 1 <= IF
    DROP 1
  ELSE
    DUP 1 - RECURSE *
  THEN ;

12 FACTORIAL .  # 479001600

# Tail recursion (optimizable)
: SUM-HELPER ( n acc -- result )
  OVER 0 = IF
    SWAP DROP
  ELSE
    SWAP DUP >R + R> 1 - SWAP RECURSE
  THEN ;

: SUM ( n -- sum )
  0 SUM-HELPER ;

100 SUM .  # 5050

# Doubly recursive (Fibonacci)
: FIBONACCI ( n -- fib(n) )
  DUP 2 < IF
  ELSE
    DUP 1 - RECURSE
    SWAP 2 - RECURSE
    +
  THEN ;

10 FIBONACCI .  # 55
```

## Performance

### Benchmarks

**Interpreted Mode:**
- ~1-5 µs per word call (AST traversal overhead)
- Good for interactive development
- No compilation delay

**JIT Mode:**
- ~10-50 ns per word call (native code)
- **100-500x speedup** for compute-intensive code
- ~1-5 ms compilation time per word

**Example: Factorial Benchmark**

```bash
# Interpreted (12!)
$ cargo run benchmarks/perf_factorial.fth
479001600
# ~5ms total execution

# JIT compiled (12!)
$ cargo run benchmarks/perf_factorial.fth --jit
479001600
# ~50µs total execution (100x faster)
```

## Testing

Run the comprehensive test suite:

```bash
# All 139 tests
cargo test

# Just Rust unit tests (133)
cargo test --lib

# Just integration tests (6)
cargo test --test forth_integration_tests

# Specific test categories
cargo test test_arithmetic    # Arithmetic tests
cargo test test_control_flow  # IF/THEN/ELSE, loops
cargo test test_recurse_jit   # Recursion in JIT mode
cargo test test_tco           # Tail call optimization
```

**Test Files:**
- `tests/run-all-tests.fth` - Main Forth test suite (runs via cargo test)
- `tests/tco_tests.fth` - Tail recursion stress tests
- `tests/recurse_tests.fth` - RECURSE word tests (both modes)

## Current Limitations

- **64-bit integers only** - No floating point yet
- **No string data type** - Strings handled as character arrays with S"
- **Limited file I/O** - INCLUDE/INCLUDED for loading Forth files only
- **String output in JIT** - `."` not yet working in JIT mode (numeric output works)
- **No exceptions** - Error handling is basic

## Dependencies

- **inkwell** (0.4) - Safe LLVM bindings for Rust
- **rustyline** (14.0) - REPL with history and line editing

Requires LLVM 18.x installed on your system.

## Project History

This learning project has evolved through several phases:

1. ✅ Basic interpreter with stack machine
2. ✅ User-defined words (`:` `;`)
3. ✅ Control flow (IF/THEN/ELSE, loops)
4. ✅ LLVM JIT compilation (Rust-based)
5. ✅ LLVM primitives for Forth compiler
6. ✅ **Self-hosting Forth compiler**
7. ✅ **64-bit migration** (i32→i64, 4→8 byte cells)
8. ✅ **Full recursion support** (RECURSE + direct recursion in JIT)
9. ✅ **Comprehensive test suite** (139 tests)

## Future Roadmap

### v0.3 - Standard Library
- [ ] More standard Forth words
- [ ] String handling words
- [ ] Double-cell arithmetic (2@, 2!, D+, D-)
- [ ] Floating point support

### v0.4 - Advanced Features
- [ ] File I/O words (OPEN-FILE, READ-FILE, etc.)
- [ ] Exception handling (CATCH, THROW)
- [ ] String output in JIT mode
- [ ] Tail call optimization in JIT

### v1.0 - Production Ready
- [ ] Full ANS Forth compliance
- [ ] Optimize generated LLVM IR
- [ ] Cross-compilation support
- [ ] Standard library in Forth

## Contributing

This is a learning project, but issues and pull requests are welcome! Areas that need work:

- Standard Forth word implementations
- Test coverage improvements
- Documentation
- Performance optimizations
- Bug fixes

## License

MIT License - See LICENSE file for details

## Acknowledgments

Built as a learning project to explore:
- Forth language design and implementation
- Rust systems programming
- LLVM code generation
- Self-hosting compiler design
- JIT compilation techniques

---

**Quarter** - Because a Forth of a dollar is 25 cents 🪙
