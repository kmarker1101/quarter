# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Quarter** is a Forth interpreter written in Rust with JIT/AOT compilation via LLVM. Three execution modes: interpreted, JIT (runtime native code), and AOT (standalone executables ~50KB).

**Current Status:** v0.2 - 156 tests passing
- ✅ Complete Forth interpreter with all core primitives
- ✅ Self-hosting Forth compiler (stdlib/compiler.fth)
- ✅ LLVM JIT compilation (100-500x speedup)
- ✅ AOT compilation to standalone executables
- ✅ Full recursion support (RECURSE + direct recursion)

**Note:** Quarter is **case-insensitive** (like gforth). `DUP`, `dup`, and `Dup` all work identically.

## Build and Run Commands

```bash
# Build and test
cargo build
cargo test                    # All 156 tests must pass
cargo clippy                  # Must have zero warnings

# Run
cargo run                                      # REPL (interpreted)
cargo run myprogram.fth                        # Run file (interpreted)
cargo run myprogram.fth --jit                  # JIT mode (100-500x faster)

# AOT compilation
cargo run -- -c myprogram.fth                  # Creates a.out
cargo run -- -c myprogram.fth -o myapp -O3     # Max optimization
cargo run -- -c myprogram.fth --keep-temps     # Keep temp files for debugging
```

## Project Structure

```
src/
  main.rs              # CLI, argument parsing, AOT compilation
  runtime.rs           # ⚠️ Single source of truth for ALL primitives
  words.rs             # Thin wrappers calling runtime.rs functions
  dictionary.rs        # Word dictionary
  lib.rs, stack.rs, ast.rs, llvm_*.rs, ast_forth.rs
build.rs               # Compiles runtime.rs separately for AOT
stdlib/                # Forth standard library
tests/                 # 148 Rust unit tests + 8 integration tests
docs/                  # Detailed documentation (see below)
```

## Adding New Primitive Words

**CRITICAL:** Dual-implementation architecture - `runtime.rs` is single source of truth for all modes.

**See [docs/adding-llvm-primitives.md](docs/adding-llvm-primitives.md) for complete guide with examples.**

**6 Steps Required:**
1. `src/runtime.rs` - Implement `quarter_*` function
2. `src/words.rs` - Add extern declaration + wrapper
3. `src/dictionary.rs` - Register in `register_primitives!` macro
4. `src/llvm_forth.rs` - Add to `register_quarter_symbols()`
5. `stdlib/compiler.fth` - Add to `DECLARE-ALL-PRIMITIVES`
6. `stdlib/compiler.fth` - (Optional) Add name mapping in `MAP-WORD-NAME`

**Test in all 3 modes** (interpreted, `--jit`, `--compile`) before committing.

**Other word types:** Compile-only words → `src/ast.rs`, Forth words → `stdlib/core.fth`

## Testing

**Required before every commit:**
1. `cargo test` - all 156 tests must pass
2. `cargo clippy` - zero warnings required
3. `cargo fmt` - auto-format code

**Add tests:**
- Rust unit tests: appropriate file in `tests/`
- Forth tests: `tests/basic_tests.fth` using `T{ -> }T` syntax

## Memory Layout (8MB total)

```
0x000000-0x00FFFF: Data Stack (64KB, 8K cells of 8 bytes each)
0x010000-0x01FFFF: Return Stack (64KB, 8K cells)
0x020000-0x7FFFFF: User Memory (~7.5MB)
```

**Cell Size:** 8 bytes (64-bit integers), little-endian

## Architecture

**Execution Modes:**
- **Interpreted**: AST evaluation (baseline speed)
- **JIT**: Runtime compilation via LLVM (100-500x faster). See [docs/jit-compilation.md](docs/jit-compilation.md)
- **AOT**: Compile to standalone binary (100-500x faster, ~50KB). See [docs/aot-compilation.md](docs/aot-compilation.md)

**Version:** Centralized in `Cargo.toml`, accessed via `const VERSION: &str = env!("CARGO_PKG_VERSION");`

## Implemented Words

Full ANS Forth core words. See [docs/word-reference.md](docs/word-reference.md) for complete reference.

## Detailed Documentation

See `docs/` for comprehensive guides:
- **[Word Reference](docs/word-reference.md)** - Complete word list
- **[JIT Compilation](docs/jit-compilation.md)** - Just-in-time compilation
- **[AOT Compilation](docs/aot-compilation.md)** - Standalone executables
- Feature docs: **[Control Flow](docs/control-flow.md)**, **[Memory](docs/memory.md)**, **[Metaprogramming](docs/metaprogramming.md)**, **[I/O](docs/io.md)**, **[Strings](docs/strings.md)**, **[Stacks](docs/stacks.md)**, **[Error Handling](docs/error-handling.md)**, **[Arithmetic](docs/arithmetic.md)**

**Developer Documentation:**
- **[Adding LLVM Primitives](docs/adding-llvm-primitives.md)** - Step-by-step implementation guide
- **[LLVM Global Strings](docs/llvm-global-strings-notes.md)** - Dual-strategy string implementation

See **[README.md](README.md)** for installation, examples, and feature overview.

## Dependencies

**System Requirements (for JIT/AOT):**
- LLVM 18.x, libzstd, libffi, libc++/libstdc++
- macOS: `brew install llvm zstd libffi`
- Ubuntu: `sudo apt install llvm-18 libzstd-dev libffi-dev`

**Rust Crates:** inkwell (LLVM bindings), rustyline (REPL)

**Note:** Interpreted mode works without LLVM.

## Key Implementation Notes

- **Case-insensitive**: All words uppercase internally
- **Self-hosting**: Forth compiler in `stdlib/compiler.fth`
- **Dual primitives**: `runtime.rs` = single source for all modes
- **TCO**: Tail-call optimization (verified to 500K calls)
- See docs for: strings, JIT redefinition handling, AOT pipeline
