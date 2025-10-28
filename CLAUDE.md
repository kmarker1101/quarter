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

**CRITICAL:** We use dual-implementation architecture for interpreter + JIT + AOT support.

### Step 1: Add to `src/runtime.rs` (single source of truth)
```rust
#[no_mangle]
pub unsafe extern "C" fn quarter_word_name(
    memory: *mut u8, sp: *mut usize, rp: *mut usize
) {
    unsafe {
        let sp_val = *sp;
        // Check bounds with check_sp_read/check_sp_write
        // Implement word using raw pointers
        *sp = new_sp_val;
    }
}
```

### Step 2: Add wrapper to `src/words.rs` (for interpreter)
```rust
pub fn word_name(stack: &mut Stack, _loop_stack: &LoopStack,
                 return_stack: &mut crate::ReturnStack,
                 memory: &mut crate::Memory) {
    unsafe {
        crate::runtime::quarter_word_name(
            memory.as_mut_ptr(),
            stack.get_sp_ptr(),
            return_stack.get_rp_ptr()
        );
    }
}
```

### Step 3: Register in `src/dictionary.rs`
```rust
"WORD-NAME" => words::word_name,
```

### Step 4: Register for JIT in `src/llvm_forth.rs` (in `register_runtime_symbols`)
```rust
crate::words::quarter_word_name,
```

**Why?** This architecture eliminates duplication - same code works for interpreter, JIT, and AOT modes.

## Adding Other Word Types

**Compile-Only Words** (IF/THEN/ELSE, LEAVE, etc.): Add to `src/ast.rs` as `AstNode` variants

**Forth Words**: Add to `stdlib/core.fth` using existing primitives

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

### Execution Modes
- **Interpreted**: AST evaluation (baseline speed)
- **JIT**: Runtime compilation to native code (100-500x faster)
- **AOT**: Compile-time to standalone executable (100-500x faster, ~50KB binaries)

### AOT Compilation Pipeline
```
Forth Source → Parse → AST → LLVM IR → Object File
                                          ↓
                      runtime.a + main.o ← Link
                                          ↓
                                   Standalone Binary
```

**Build artifacts:** Temp files in `/tmp/quarter_build_<PID>/` (auto-cleaned unless `--keep-temps`)

### Version Management
Centralized in `Cargo.toml`, accessed via `const VERSION: &str = env!("CARGO_PKG_VERSION");`

## Implemented Words (Quick Reference)

**Core:** `+`, `-`, `*`, `/`, `MOD`, `DUP`, `DROP`, `SWAP`, `OVER`, `ROT`, `!`, `@`, `C!`, `C@`
**Comparison:** `<`, `>`, `=`, `0=`, `0<`, `0>`
**Control Flow:** `IF/THEN/ELSE`, `BEGIN/UNTIL`, `BEGIN/WHILE/REPEAT`, `DO/LOOP`, `+LOOP`, `LEAVE`, `EXIT`, `RECURSE`
**I/O:** `.`, `.S`, `CR`, `EMIT`, `KEY`, `TYPE`, `."`, `S"`
**Memory:** `HERE`, `ALLOT`, `,`, `VARIABLE`, `CONSTANT`, `CREATE`, `ALIGNED`, `ALIGN`, `FILL`
**Metaprogramming:** `EXECUTE`, `'`, `[']`, `FIND`, `IMMEDIATE`, `CHAR`, `[CHAR]`
**Error Handling:** `ABORT`, `ABORT"`
**File Loading:** `INCLUDE`, `INCLUDED`

## Detailed Documentation

See `docs/` for comprehensive guides:
- **[AOT Compilation](docs/aot-compilation.md)** - Standalone executables, optimization levels, troubleshooting
- **[Control Flow](docs/control-flow.md)**, **[Memory](docs/memory.md)**, **[Metaprogramming](docs/metaprogramming.md)**
- **[I/O](docs/io.md)**, **[Stacks](docs/stacks.md)**, **[Error Handling](docs/error-handling.md)**, **[Arithmetic](docs/arithmetic.md)**

See **[README.md](README.md)** for installation, examples, and feature overview.

## Dependencies

**System Requirements (for JIT/AOT):**
- LLVM 18.x, libzstd, libffi, libc++/libstdc++
- macOS: `brew install llvm zstd libffi`
- Ubuntu: `sudo apt install llvm-18 libzstd-dev libffi-dev`

**Rust Crates:** inkwell (LLVM bindings), rustyline (REPL)

**Note:** Interpreted mode works without LLVM.

## Key Implementation Notes

- **Case-insensitive**: All words converted to uppercase
- **Compile-time validation**: All referenced words must exist
- **TCO**: Tail-call optimization (verified to 500,000 calls)
- **Dual primitives**: `runtime.rs` = single source, `words.rs` = wrappers, `build.rs` = separate compilation
- **Clean builds**: Auto-cleanup of temp files after AOT compilation
- **Self-hosting**: Forth compiler written in Forth (stdlib/compiler.fth)
