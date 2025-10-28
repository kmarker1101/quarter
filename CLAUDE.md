# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Note:** Quarter is **case-insensitive** (like gforth). `DUP`, `dup`, and `Dup` all work identically.

## Project Overview

**quarter** is a Forth interpreter written in Rust with a **self-hosting JIT compiler** that compiles Forth code to native machine code via LLVM. It supports three execution modes: interpreted, JIT (runtime native code), and AOT (standalone executables).

**Current Status:** v2 Complete ✅
- ✅ Complete Forth interpreter with all core primitives
- ✅ Self-hosting Forth compiler (stdlib/compiler.fth - 666 lines)
- ✅ LLVM JIT compilation (100-500x speedup)
- ✅ AOT compilation to standalone executables
- ✅ Full recursion support (RECURSE + direct recursion)
- ✅ 153 tests passing (145 unit + 8 integration)

## Development Workflow

**CRITICAL**: This project follows a learn-by-typing workflow:
- The developer types ALL code themselves - never update files directly
- Display code snippets for the developer to type in
- Only make changes to files when explicitly instructed
- Guide and explain, but don't execute

## Build and Run Commands

```bash
# Build
cargo build

# Run REPL
cargo run

# Run file (interpreted)
cargo run myprogram.fth

# Run with JIT compilation
cargo run myprogram.fth --jit

# AOT compile to standalone executable
cargo run -- --compile myprogram.fth
cargo run -- --compile myprogram.fth -o mybinary
cargo run -- -c myprogram.fth -o myapp -O3

# Tests
cargo test                    # All tests
cargo test --lib              # Unit tests only
cargo test test_name          # Specific test

# Code quality (REQUIRED before every commit)
cargo clippy                  # Must pass with zero warnings
cargo fmt                     # Auto-format code
```

## Project Structure

```
src/
  main.rs              # CLI
  lib.rs               # Parser, file loading
  stack.rs             # Data stack (64-bit cells in memory)
  dictionary.rs        # Word dictionary
  words.rs             # Built-in primitive words
  ast.rs               # Abstract Syntax Tree
  llvm_codegen.rs      # Rust-based LLVM code generator
  llvm_forth.rs        # LLVM primitives for Forth compiler
  ast_forth.rs         # AST inspection for Forth compiler

stdlib/
  core.fth             # Core Forth standard library
  test-framework.fth   # Unit test framework
  repl.fth             # Repl

tests/
  *.rs                 # Rust unit tests (145 tests)
  basic_tests.fth      # Main Forth test suite (93 tests)
  forth_integration_tests.rs  # Integration tests (8 tests)

docs/                  # Detailed documentation (see below)
```

## Implemented Words (Quick Reference)

**Arithmetic**: `+`, `-`, `*`, `/`, `/MOD`, `MOD`, `NEGATE`, `ABS`, `MIN`, `MAX`, `1+`, `1-`, `2*`, `2/`
**Comparison**: `<`, `>`, `=`, `<>`, `<=`, `>=`, `0=`, `0<`, `0>`
**Stack**: `DUP`, `DROP`, `SWAP`, `OVER`, `ROT`, `PICK`, `DEPTH`
**Return Stack**: `>R`, `R>`, `R@`
**Memory**: `!`, `@`, `C!`, `C@`, `+!`, `HERE`, `ALLOT`, `,`, `ALIGNED`, `ALIGN`, `FILL`
**Bitwise**: `AND`, `OR`, `XOR`, `INVERT`, `LSHIFT`, `RSHIFT`
**I/O**: `.`, `.S`, `CR`, `EMIT`, `KEY`, `SPACE`, `TYPE`
**Strings**: `."`, `S"`
**Control Flow**: `IF/THEN/ELSE`, `BEGIN/UNTIL`, `BEGIN/WHILE/REPEAT`, `DO/LOOP`, `+LOOP`, `LEAVE`, `EXIT`, `RECURSE`
**Metaprogramming**: `EXECUTE`, `'`, `[']`, `CHAR`, `[CHAR]`, `COUNT`, `FIND`, `IMMEDIATE`
**Error Handling**: `ABORT`, `ABORT"`, `CATCH`, `THROW`
**Definition**: `:`, `;`, `VARIABLE`, `CONSTANT`, `CREATE`
**File Loading**: `INCLUDE`, `INCLUDED`

## Adding New Words

### 1. Primitive Words (Rust - src/words.rs)
```rust
pub fn word_name(stack: &mut Stack, _loop_stack: &LoopStack,
                 _return_stack: &mut crate::ReturnStack,
                 memory: &mut crate::Memory) {
    // Implementation
}
```
Then register in `src/dictionary.rs`:
```rust
"WORD-NAME" => words::word_name,
```

### 2. Compile-Only Words (AST - src/ast.rs)
Add variant to `AstNode` enum, implement validation and execution.

### 3. Forth Words (stdlib/core.fth)
```forth
: WORD-NAME ( stack-effect )
  \ Implementation using existing words
;
```

## Testing

**Before every commit:**
1. Run `cargo test` - all 153 tests must pass
2. Run `cargo clippy` - zero warnings required
3. Run Forth tests: `cargo run tests/basic_tests.fth`

**Adding tests:**
- Rust unit tests: appropriate file in `tests/`
- Forth tests: add to `tests/basic_tests.fth` using `T{ -> }T` syntax

## Architecture Notes

**Memory Layout** (8MB total):
- `0x000000-0x00FFFF`: Data Stack (64KB, 8K cells)
- `0x010000-0x01FFFF`: Return Stack (64KB, 8K cells)
- `0x020000-0x7FFFFF`: User Memory (~7.5MB)

**Cell Size**: 8 bytes (64-bit integers)
**Endianness**: Little-endian

## Detailed Documentation

For detailed examples, tutorials, and implementation guides:

- **[Control Flow](docs/control-flow.md)** - IF/THEN/ELSE, loops, LEAVE, EXIT, RECURSE
- **[Memory Operations](docs/memory.md)** - Memory access, allocation, alignment
- **[Metaprogramming](docs/metaprogramming.md)** - EXECUTE, TICK, FIND, IMMEDIATE
- **[I/O Operations](docs/io.md)** - Character I/O, strings, output
- **[Stack Operations](docs/stacks.md)** - Data stack and return stack
- **[Error Handling](docs/error-handling.md)** - ABORT, ABORT", CATCH, THROW
- **[Arithmetic](docs/arithmetic.md)** - Math operations and comparisons

## Git Workflow

**Committing:**
```bash
git add <files>
git commit -m "Brief description

- Detailed changes
- Testing notes


```


## Dependencies

**Rust Crates:**
- **inkwell** (0.4) - LLVM bindings
- **rustyline** (14.0) - REPL with history

**System Requirements:**
- **LLVM 18.x** - Required for JIT and AOT compilation
- **libzstd** - Compression library (used by LLVM)
- **libffi** - Foreign Function Interface library
- **libc++** (macOS) or **libstdc++** (Linux) - C++ standard library
- **libz** - Compression library

**Installation:**
- macOS: `brew install llvm zstd libffi`
- Ubuntu/Debian: `sudo apt install llvm-18 libzstd-dev libffi-dev`

**Note:** Interpreted mode works without LLVM, but JIT and AOT modes require these libraries.

## Key Implementation Details

- **Case-insensitive**: All word names converted to uppercase
- **Compile-time validation**: Word definitions validate all referenced words exist
- **TCO**: Tail-call optimization for recursive words
- **JIT Compilation**: Self-hosting Forth compiler in `stdlib/compiler.fth`
- **AOT Compilation**: Compiles Forth programs to standalone native executables via LLVM
- **Test Framework**: `T{ -> }T` syntax in `stdlib/test-framework.fth`
