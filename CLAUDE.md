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

**See [docs/adding-llvm-primitives.md](docs/adding-llvm-primitives.md) for complete step-by-step guide.**

### Quick Summary (6 steps required)

1. **`src/runtime.rs`** - Implement `quarter_*` function (single source of truth)
2. **`src/words.rs`** - Add extern declaration + wrapper for interpreted mode
3. **`src/dictionary.rs`** - Register in `register_primitives!` macro
4. **`src/llvm_forth.rs`** - Add to `register_quarter_symbols()` for AOT linking
5. **`stdlib/compiler.fth`** - Add to `DECLARE-ALL-PRIMITIVES` (byte-by-byte ASCII)
6. **`stdlib/compiler.fth`** - (Optional) Add name mapping in `MAP-WORD-NAME` for special characters

### Example: Simple primitive

```rust
// Step 1: src/runtime.rs
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_word_name(
    memory: *mut u8, sp: *mut usize, rp: *mut usize
) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) { return; }
        let val = (memory.add(sp_val - 8) as *const i64).read_unaligned();
        // ... implement logic ...
        let result_addr = memory.add(sp_val - 8) as *mut i64;
        *result_addr = result;
    }
}

// Step 2: src/words.rs - extern + wrapper
unsafe extern "C" {
    pub fn quarter_word_name(memory: *mut u8, sp: *mut usize, rp: *mut usize);
}
pub fn word_name(stack: &mut Stack, ...) { /* wrapper impl */ }

// Step 3: src/dictionary.rs
"WORD-NAME" => words::word_name,

// Step 4: src/llvm_forth.rs
crate::words::quarter_word_name,

// Step 5: stdlib/compiler.fth (DECLARE-ALL-PRIMITIVES)
\ ASCII bytes for "quarter_word_name" then DECLARE-PRIMITIVE

// Step 6: stdlib/compiler.fth (MAP-WORD-NAME) - only if needed
\ Custom mapping for special characters like "-TRAILING"
```

**Why?** This architecture eliminates duplication - same code works for interpreter, JIT, and AOT modes.

**Testing:** Test in all three modes (interpreted, `--jit`, `--compile`) before committing.

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
**I/O:** `.`, `.S`, `CR`, `EMIT`, `KEY`, `TYPE`, `."`, `S"`, `C"`
**Strings:** `COMPARE`, `-TRAILING`, `SEARCH`, `/STRING`, `ERASE`
**Memory:** `HERE`, `ALLOT`, `,`, `VARIABLE`, `CONSTANT`, `CREATE`, `ALIGNED`, `ALIGN`, `FILL`
**Metaprogramming:** `EXECUTE`, `'`, `[']`, `FIND`, `IMMEDIATE`, `CHAR`, `[CHAR]`
**Error Handling:** `ABORT`, `ABORT"`
**File Loading:** `INCLUDE`, `INCLUDED`

## Detailed Documentation

See `docs/` for comprehensive guides:
- **[AOT Compilation](docs/aot-compilation.md)** - Standalone executables, optimization levels, troubleshooting
- **[Control Flow](docs/control-flow.md)**, **[Memory](docs/memory.md)**, **[Metaprogramming](docs/metaprogramming.md)**
- **[I/O](docs/io.md)**, **[Strings](docs/strings.md)**, **[Stacks](docs/stacks.md)**, **[Error Handling](docs/error-handling.md)**, **[Arithmetic](docs/arithmetic.md)**

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

- **Case-insensitive**: All words converted to uppercase
- **Compile-time validation**: All referenced words must exist
- **TCO**: Tail-call optimization (verified to 500,000 calls)
- **Dual primitives**: `runtime.rs` = single source, `words.rs` = wrappers, `build.rs` = separate compilation
- **Clean builds**: Auto-cleanup of temp files after AOT compilation
- **Self-hosting**: Forth compiler written in Forth (stdlib/compiler.fth)
- **Dual-strategy strings**: JIT mode uses HERE-based allocation, AOT mode uses LLVM global constants (see [docs/llvm-global-strings-notes.md](docs/llvm-global-strings-notes.md))
