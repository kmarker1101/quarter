# Adding LLVM Primitives to Quarter

This guide explains how to add a new primitive word that works in all three execution modes: interpreted, JIT, and AOT.

## Overview

Quarter uses a **dual-implementation architecture** where primitives are implemented once in `runtime.rs` and used by all three modes:

1. **Interpreted mode:** Calls Rust implementation directly via wrapper
2. **JIT mode:** LLVM generates call to `quarter_*` function at runtime
3. **AOT mode:** Links against `quarter_*` function in static library

## Step-by-Step Guide

### Step 1: Implement in `src/runtime.rs`

Create the actual implementation with signature:
```rust
unsafe extern "C" fn(memory: *mut u8, sp: *mut usize, rp: *mut usize)
```

**Example: COMPARE primitive**

```rust
/// COMPARE ( c-addr1 u1 c-addr2 u2 -- n )
/// Compare two strings, return -1 (less), 0 (equal), or 1 (greater)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_compare(
    memory: *mut u8,
    sp: *mut usize,
    _rp: *mut usize
) {
    unsafe {
        let sp_val = *sp;
        
        // Check stack has enough values (4 cells = 32 bytes)
        if !check_sp_read(sp_val, 32) {
            return;
        }

        // Pop parameters (little-endian, 8 bytes per cell)
        let u2 = (memory.add(sp_val - 8) as *const i64).read_unaligned();
        let addr2 = (memory.add(sp_val - 16) as *const i64).read_unaligned();
        let u1 = (memory.add(sp_val - 24) as *const i64).read_unaligned();
        let addr1 = (memory.add(sp_val - 32) as *const i64).read_unaligned();

        // Validate inputs
        if u1 < 0 || u2 < 0 {
            return;
        }

        let u1 = u1 as usize;
        let u2 = u2 as usize;
        let addr1 = addr1 as usize;
        let addr2 = addr2 as usize;

        // Implement logic
        let min_len = u1.min(u2);
        for i in 0..min_len {
            let byte1 = *memory.add(addr1 + i);
            let byte2 = *memory.add(addr2 + i);
            if byte1 < byte2 {
                // Write result and adjust stack pointer
                let result_addr = memory.add(sp_val - 32) as *mut i64;
                *result_addr = -1;
                *sp = sp_val - 24;  // Consumed 4, produced 1
                return;
            } else if byte1 > byte2 {
                let result_addr = memory.add(sp_val - 32) as *mut i64;
                *result_addr = 1;
                *sp = sp_val - 24;
                return;
            }
        }

        // Strings equal, compare lengths
        let result = if u1 < u2 { -1 } else if u1 > u2 { 1 } else { 0 };
        let result_addr = memory.add(sp_val - 32) as *mut i64;
        *result_addr = result;
        *sp = sp_val - 24;
    }
}
```

**Key points:**
- Use `#[unsafe(no_mangle)]` to preserve function name for linking
- Use `check_sp_read(sp_val, bytes)` to validate stack depth
- Stack grows upward: `sp_val - 8` is top of stack
- Read cells with `read_unaligned()` (8-byte i64)
- Write result at appropriate stack position
- Update stack pointer: `*sp = new_value`
- Each cell is 8 bytes (64-bit)

### Step 2: Add extern declaration in `src/words.rs`

Add to the `unsafe extern "C"` block:

```rust
unsafe extern "C" {
    // ... existing declarations ...
    
    // String operations
    pub fn quarter_compare(memory: *mut u8, sp: *mut usize, rp: *mut usize);
    pub fn quarter_minus_trailing(memory: *mut u8, sp: *mut usize, rp: *mut usize);
    pub fn quarter_search(memory: *mut u8, sp: *mut usize, rp: *mut usize);
}
```

Then create high-level wrapper for interpreted mode:

```rust
pub fn compare(
    stack: &mut Stack,
    _loop_stack: &LoopStack,
    _return_stack: &mut crate::ReturnStack,
    memory: &mut crate::Memory,
) {
    // Pop parameters
    let u2 = match stack.pop(memory) {
        Some(v) => v,
        None => {
            eprintln!("Stack underflow in COMPARE");
            return;
        }
    };
    // ... pop addr2, u1, addr1 ...

    // Validate
    if u1 < 0 || u2 < 0 {
        eprintln!("COMPARE: invalid length");
        return;
    }

    // Implement logic using Stack abstraction
    let u1 = u1 as usize;
    let u2 = u2 as usize;
    let addr1 = addr1 as usize;
    let addr2 = addr2 as usize;

    // ... comparison logic ...

    // Push result
    stack.push(result, memory);
}
```

**Note:** The wrapper implementation should match the runtime implementation's behavior exactly. Both are needed because:
- Wrapper is used in interpreted mode (uses Stack abstraction)
- Runtime is used in JIT/AOT modes (direct memory access)

### Step 3: Register in `src/dictionary.rs`

Add to the `register_primitives!` macro call in `Dictionary::new()`:

```rust
register_primitives!(dict,
    // ... existing primitives ...
    "TYPE" => words::type_word,
    "COMPARE" => words::compare,
    "-TRAILING" => words::minus_trailing,
    "SEARCH" => words::search,
    "KEY" => words::key,
    // ... more primitives ...
```

This makes the word callable by name from Forth code in interpreted mode.

### Step 4: Register symbol in `src/llvm_forth.rs`

Add to the `register_quarter_symbols()` function:

```rust
fn register_quarter_symbols() -> usize {
    let symbols = symbol_array!(
        // ... existing symbols ...
        crate::words::quarter_comma,

        // String operations
        crate::words::quarter_compare,
        crate::words::quarter_minus_trailing,
        crate::words::quarter_search,
    );
    symbols[0]
}
```

This forces the linker to include the symbols in the final binary for AOT compilation.

### Step 5: Declare primitive in `stdlib/compiler.fth`

Add to the `DECLARE-ALL-PRIMITIVES` word (around line 2604):

```forth
    \ String - quarter_compare
    113 COMPILER-SCRATCH 0 + C!   \ q
    117 COMPILER-SCRATCH 1 + C!   \ u
    97  COMPILER-SCRATCH 2 + C!   \ a
    114 COMPILER-SCRATCH 3 + C!   \ r
    116 COMPILER-SCRATCH 4 + C!   \ t
    101 COMPILER-SCRATCH 5 + C!   \ e
    114 COMPILER-SCRATCH 6 + C!   \ r
    95  COMPILER-SCRATCH 7 + C!   \ _
    99  COMPILER-SCRATCH 8 + C!   \ c
    111 COMPILER-SCRATCH 9 + C!   \ o
    109 COMPILER-SCRATCH 10 + C!  \ m
    112 COMPILER-SCRATCH 11 + C!  \ p
    97  COMPILER-SCRATCH 12 + C!  \ a
    114 COMPILER-SCRATCH 13 + C!  \ r
    101 COMPILER-SCRATCH 14 + C!  \ e
    COMPILER-SCRATCH 15 DECLARE-PRIMITIVE
```

**ASCII conversion helper:**
```bash
echo -n "quarter_compare" | od -An -tu1 -w1
```

This tells the LLVM module that `quarter_compare` is an external function with signature `void(i8*, i64*, i64*)`.

### Step 6 (Optional): Add name mapping in `stdlib/compiler.fth`

**Only needed if:**
- Word name has special characters (e.g., `-TRAILING`)
- Default lowercase mapping doesn't work

Add to `MAP-WORD-NAME` function (around line 103):

```forth
    \ Check for -TRAILING (45, 84, 82, 65, 73, 76, 73, 78, 71)
    DUP 9 = IF
        OVER C@ 45 = 2 PICK 1 + C@ 84 = AND
        2 PICK 2 + C@ 82 = AND 2 PICK 3 + C@ 65 = AND
        2 PICK 4 + C@ 73 = AND 2 PICK 5 + C@ 76 = AND
        2 PICK 6 + C@ 73 = AND 2 PICK 7 + C@ 78 = AND
        2 PICK 8 + C@ 71 = AND IF
            DROP DROP
            \ Write "quarter_minus_trailing" (22 chars)
            113 COMPILER-SCRATCH  0 + C!  \ q
            117 COMPILER-SCRATCH  1 + C!  \ u
            97  COMPILER-SCRATCH  2 + C!  \ a
            114 COMPILER-SCRATCH  3 + C!  \ r
            116 COMPILER-SCRATCH  4 + C!  \ t
            101 COMPILER-SCRATCH  5 + C!  \ e
            114 COMPILER-SCRATCH  6 + C!  \ r
            95  COMPILER-SCRATCH  7 + C!  \ _
            109 COMPILER-SCRATCH  8 + C!  \ m
            105 COMPILER-SCRATCH  9 + C!  \ i
            110 COMPILER-SCRATCH 10 + C!  \ n
            117 COMPILER-SCRATCH 11 + C!  \ u
            115 COMPILER-SCRATCH 12 + C!  \ s
            95  COMPILER-SCRATCH 13 + C!  \ _
            116 COMPILER-SCRATCH 14 + C!  \ t
            114 COMPILER-SCRATCH 15 + C!  \ r
            97  COMPILER-SCRATCH 16 + C!  \ a
            105 COMPILER-SCRATCH 17 + C!  \ i
            108 COMPILER-SCRATCH 18 + C!  \ l
            105 COMPILER-SCRATCH 19 + C!  \ i
            110 COMPILER-SCRATCH 20 + C!  \ n
            103 COMPILER-SCRATCH 21 + C!  \ g
            COMPILER-SCRATCH 22 EXIT
        THEN
    THEN
```

**Default mapping:** `MAP-WORD-NAME` converts word names to lowercase with `quarter_` prefix.
- `DUP` → `quarter_dup`
- `SWAP` → `quarter_swap`
- `@` → `quarter_fetch` (special case already exists)

**Need custom mapping for:**
- `-TRAILING` → `quarter_minus_trailing` (hyphen becomes `minus_`)
- `0=` → `quarter_0eq` (already exists)
- Words with special characters

## Testing

### Test in interpreted mode:

```bash
cargo build
cargo run
```

```forth
S" ABC" S" ABC" COMPARE .  \ Should print: 0
```

### Test in JIT mode:

```bash
cargo run --jit
```

```forth
: TEST S" ABC" S" ABC" COMPARE . CR ;
TEST  \ Should print: 0
```

### Test in AOT mode:

Create test file:
```forth
: MAIN
  S" ABC" S" ABC" COMPARE . CR ;
```

Compile and run:
```bash
cargo build
target/debug/quarter --compile test.fth -o test -O2
./test  \ Should print: 0
```

## Common Patterns

### Stack manipulation (consume N, produce M)

```rust
// Pop N values
let sp_val = *sp;
if !check_sp_read(sp_val, N * 8) {
    return;
}

let val1 = (memory.add(sp_val - 8) as *const i64).read_unaligned();
let val2 = (memory.add(sp_val - 16) as *const i64).read_unaligned();
// ... pop more values ...

// Compute result
let result = /* ... */;

// Write M results
let result_addr = memory.add(sp_val - (N * 8)) as *mut i64;
*result_addr = result;
// If M > 1, write more results at result_addr + 8, result_addr + 16, etc.

// Adjust stack pointer
*sp = sp_val - (N * 8) + (M * 8);
```

### Bounds checking

```rust
if addr >= 8388608 - 3 {  // 8MB - 4 bytes for cell
    eprintln!("Memory access out of bounds");
    return;
}
```

Use helper functions in `runtime.rs`:
- `check_sp_read(sp: usize, bytes: usize) -> bool`
- `check_sp_write(sp: usize, bytes: usize) -> bool`

### Error handling

**Runtime functions:** Return silently on error (stack underflow, bounds check failure). Interpreter checks for errors at higher level.

**Wrapper functions:** Use `eprintln!()` to print error messages, then return early.

## Architecture Summary

```
┌─────────────────────────────────────────────────────────┐
│                   FORTH USER CODE                        │
│                 : WORD ... COMPARE ... ;                 │
└─────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   INTERPRETED           JIT                AOT
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │ wrapper  │      │ LLVM IR  │      │ LLVM IR  │
  │ (words)  │      │ codegen  │      │ codegen  │
  └──────────┘      └──────────┘      └──────────┘
        │                  │                  │
        ▼                  ▼                  ▼
  ┌──────────────────────────────────────────────┐
  │    quarter_compare (runtime.rs)              │
  │    Single implementation - no duplication    │
  └──────────────────────────────────────────────┘
```

## Checklist

When adding a new LLVM primitive:

- [ ] Step 1: Implement `quarter_*` function in `src/runtime.rs`
  - [ ] Use `#[unsafe(no_mangle)]`
  - [ ] Signature: `unsafe extern "C" fn(memory: *mut u8, sp: *mut usize, rp: *mut usize)`
  - [ ] Validate inputs (bounds checking)
  - [ ] Implement logic
  - [ ] Update stack pointer correctly

- [ ] Step 2: Add declarations to `src/words.rs`
  - [ ] `extern "C"` declaration in unsafe block
  - [ ] High-level wrapper function for interpreted mode

- [ ] Step 3: Register in `src/dictionary.rs`
  - [ ] Add to `register_primitives!` macro

- [ ] Step 4: Register symbol in `src/llvm_forth.rs`
  - [ ] Add to `symbol_array!` in `register_quarter_symbols()`

- [ ] Step 5: Declare in `stdlib/compiler.fth`
  - [ ] Add to `DECLARE-ALL-PRIMITIVES` word
  - [ ] Byte-by-byte ASCII construction

- [ ] Step 6: (Optional) Name mapping
  - [ ] Add custom mapping to `MAP-WORD-NAME` if needed

- [ ] Test in all three modes
  - [ ] Interpreted: `cargo run`
  - [ ] JIT: `cargo run --jit`
  - [ ] AOT: `cargo run --compile`

## Examples in Codebase

**Simple primitive (single input, single output):**
- `quarter_abs` in `src/runtime.rs:328`
- `quarter_negate` in `src/runtime.rs:308`

**Multiple inputs, single output:**
- `quarter_add` in `src/runtime.rs:164`
- `quarter_lt` in `src/runtime.rs:412`

**Complex multi-parameter:**
- `quarter_compare` in `src/runtime.rs:1069` (4 inputs → 1 output)
- `quarter_search` in `src/runtime.rs:1222` (4 inputs → 3 outputs)

**Memory access:**
- `quarter_fetch` in `src/runtime.rs:671`
- `quarter_store` in `src/runtime.rs:641`

## Troubleshooting

### "Undefined word" in interpreted mode
→ Check `src/dictionary.rs` registration

### "Invalid function handle: 0" in JIT mode
→ Check `MAP-WORD-NAME` in `stdlib/compiler.fth`

### Linker error in AOT mode
→ Check `register_quarter_symbols()` in `src/llvm_forth.rs`

### "LLVM-BUILD-CALL error" in JIT/AOT
→ Check `DECLARE-ALL-PRIMITIVES` in `stdlib/compiler.fth`

### Segfault or memory corruption
→ Check stack pointer arithmetic and bounds checking

### Wrong results
→ Ensure runtime and wrapper implementations match
→ Check little-endian byte order
→ Verify stack effect (inputs consumed, outputs produced)
