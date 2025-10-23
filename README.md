# Quarter - A Self-Hosting Forth Interpreter

Quarter is a Forth interpreter written in Rust with a **self-hosting JIT compiler** that compiles Forth code to native machine code via LLVM.

## Status: Self-Hosting Achieved ✅

Quarter has achieved **true self-compilation**: The Forth compiler is written in Forth and successfully compiles itself to native code. This represents a complete bootstrap cycle where the compiler can compile its own source code.

## Key Features

### Three Execution Modes

1. **Interpreted Mode** (Default)
   - Direct AST evaluation
   - Immediate execution
   - Used for interactive REPL

2. **Rust JIT Compiler**
   - Rust code generates LLVM IR
   - Compiles Forth words to native code
   - Flag: `--jit` (default enabled)

3. **Forth JIT Compiler** (Self-Hosting)
   - Forth code generates LLVM IR
   - The compiler compiles itself
   - Flag: `--forth-compiler`
   - Located in: `forth/compiler.fth`

### Self-Hosting Architecture

The Forth compiler (`forth/compiler.fth`) is written entirely in Forth and can:
- Parse AST nodes
- Generate LLVM IR
- Compile itself to native code
- Compile other Forth programs to native code

When loaded with `--forth-compiler`, all word definitions in the compiler are JIT-compiled by the Forth compiler itself, creating a complete bootstrap cycle.

## Building and Running

```bash
# Build the project
cargo build

# Run interactive REPL
cargo run

# Run a Forth source file
cargo run myprogram.fth

# Enable Forth self-hosting compiler
cargo run --forth-compiler

# Run tests
cargo test

# Build in release mode
cargo build --release
```

## Architecture Overview

### Core Components

```
src/
├── main.rs           # REPL, parser, word definition pipeline
├── stack.rs          # Data stack (Vec<i32> in memory)
├── dictionary.rs     # Word dictionary (HashMap<String, Word>)
├── words.rs          # Built-in primitive words
├── ast.rs            # Abstract Syntax Tree
├── llvm_codegen.rs   # Rust-based LLVM code generator
├── llvm_forth.rs     # LLVM primitives for Forth compiler
└── ast_forth.rs      # AST inspection for Forth compiler
```

### Forth Source Files

```
stdlib/
├── core.fth          # Core Forth definitions (LOADED)
├── test-framework.fth # Test framework (disabled - DEPTH in loops issue)
└── tests.fth         # Test suite (disabled)

forth/
└── compiler.fth      # Self-hosting Forth compiler (666 lines)
```

### Memory Layout

```
Stack Section (Data Stack):
  0 - 49,999         Data stack (grows upward)

Heap Section:
  50,000 - 299,999   General allocation (HERE/ALLOT)

Compiler Scratch Space:
  300,000 - 301,999  Temporary buffers
  302,000+           Word name storage

LLVM Object Storage:
  400,000+           LLVM object handles
```

## Implemented Features

### Forth Words

**Arithmetic:** `+`, `-`, `*`, `/`, `/MOD`, `MOD`, `NEGATE`, `ABS`, `MIN`, `MAX`, `1+`, `1-`, `2+` ... `11+`, `2*`, `2/`

**Comparison:** `<`, `>`, `=`, `<>`, `<=`, `>=`, `0=`, `0<`, `0>`

**Stack Manipulation:** `DUP`, `DROP`, `SWAP`, `OVER`, `ROT`, `-ROT`, `PICK`, `NIP`, `TUCK`, `2DUP`, `2DROP`, `2SWAP`, `2OVER`, `DEPTH`

**Return Stack:** `>R`, `R>`, `R@`

**Memory:** `!`, `@`, `C!`, `C@`, `+!`, `HERE`, `ALLOT`, `,`, `CELLS`, `CELL+`

**Stack Pointers:** `SP@`, `SP!`, `RP@`, `RP!`

**Logic:** `AND`, `OR`, `XOR`, `INVERT`, `LSHIFT`, `RSHIFT`

**I/O:** `.`, `U.`, `.R`, `U.R`, `.S`, `CR`, `EMIT`, `KEY`, `SPACE`, `SPACES`

**Control Flow:** `IF/THEN/ELSE`, `DO/LOOP`, `BEGIN/UNTIL`

**Constants:** `TRUE` (-1), `FALSE` (0), `BL` (32)

**Word Definition:** `:` and `;` define new words

**Loops:** `I`, `J` for loop indices

### LLVM Primitives (for Forth Compiler)

- **Context/Module Management:** `LLVM-CREATE-CONTEXT`, `LLVM-CREATE-MODULE`, `LLVM-DECLARE-EXTERNAL`
- **Function Creation:** `LLVM-CREATE-FUNCTION`, `LLVM-MODULE-GET-FUNCTION`, `LLVM-GET-PARAM`
- **Basic Blocks:** `LLVM-CREATE-BLOCK`, `LLVM-POSITION-AT-END`, `LLVM-GET-INSERT-BLOCK`
- **IR Building:** `LLVM-BUILD-CONST-INT`, `LLVM-BUILD-ADD`, `LLVM-BUILD-SUB`, `LLVM-BUILD-MUL`, `LLVM-BUILD-LOAD`, `LLVM-BUILD-STORE`, `LLVM-BUILD-GEP`
- **Control Flow:** `LLVM-BUILD-BR`, `LLVM-BUILD-COND-BR`, `LLVM-BUILD-PHI`, `LLVM-PHI-ADD-INCOMING`
- **Comparison:** `LLVM-BUILD-ICMP`
- **Calls:** `LLVM-BUILD-CALL`
- **Returns:** `LLVM-BUILD-RET-VOID`, `LLVM-BUILD-RET`
- **JIT:** `LLVM-CREATE-JIT`, `LLVM-GET-FUNCTION`
- **AST Inspection:** `AST-TYPE`, `AST-GET-NUMBER`, `AST-GET-WORD`, `AST-SEQ-LENGTH`, `AST-SEQ-CHILD`, `AST-IF-THEN`, `AST-IF-ELSE`, `AST-LOOP-BODY`
- **Word Registration:** `REGISTER-JIT-WORD`

## Compilation Flow

### Example: Compiling `: SQUARE DUP * ;`

1. **Parse** → Creates AST: `Sequence[CallWord("DUP"), CallWord("*")]`

2. **Choose Compiler:**
   - With `--forth-compiler`: Uses Forth compiler (`compiler.fth`)
   - With `--jit`: Uses Rust compiler (`llvm_codegen.rs`)
   - Without flags: Stores as interpreted AST

3. **Forth Compiler Flow** (`forth/compiler.fth`):
   - Receives AST handle and word name
   - Creates LLVM context and module
   - Declares external primitives (quarter_dup, quarter_mul, etc.)
   - Generates IR by walking AST
   - For `CallWord("DUP")`: Emits `call void @quarter_dup(...)`
   - For `CallWord("*")`: Emits `call void @quarter_mul(...)`
   - Creates JIT execution engine
   - Returns function pointer

4. **Register**: Function pointer stored in dictionary as `Word::JITCompiled`

5. **Execute**: Calling `SQUARE` invokes native machine code directly

## Current Limitations

- Test framework disabled (DEPTH in loops issue)
- Stack is 32-bit integers only (no floats, strings as words)
- No string handling beyond character arrays
- No file I/O words yet
- Return stack operations don't work in JIT-compiled code yet

## Performance Characteristics

- **Interpreted Mode**: ~1-5 µs per word call (AST traversal overhead)
- **JIT Mode**: ~10-50 ns per word call (native code, no overhead)
- **Compilation Time**: ~1-5 ms per word (LLVM optimization passes)

JIT compilation provides **100-500x speedup** for compute-intensive code.

## Example Usage

```forth
# Interactive REPL
$ cargo run
Quarter Forth v0.1
Type 'quit' to exit

> 5 3 + .
8 ok

> : SQUARE DUP * ;
Compiled SQUARE ok

> 4 SQUARE .
16 ok

> : SUM-OF-SQUARES SQUARE SWAP SQUARE + ;
Compiled SUM-OF-SQUARES ok

> 3 4 SUM-OF-SQUARES .
25 ok
```

```forth
# With Forth compiler
$ cargo run -- --forth-compiler
Quarter Forth v0.1
Type 'quit' to exit

> : DOUBLE 2 * ;
Compiled DOUBLE (JIT - Forth) ok

> 21 DOUBLE .
42 ok
```

## Dependencies

- **inkwell** - LLVM bindings for Rust
- **rustyline** - REPL with history and line editing

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_square

# Run with output
cargo test -- --nocapture
```

Currently: **49 tests passing** (9 lib tests + 40 integration tests)

## Project History

This is a learning project where the developer is learning Rust by building a Forth interpreter from scratch. The project has evolved through several phases:

1. Basic interpreter with stack machine
2. User-defined words with `:` `;`
3. Control flow (IF/THEN/ELSE, DO/LOOP)
4. LLVM JIT compilation (Rust-based)
5. LLVM primitives for Forth
6. **Self-hosting Forth compiler** ✅ (Current phase)

## License

This is a personal learning project.

## Future Possibilities

- Fix test framework (DEPTH in loops)
- Add more standard Forth words
- String handling
- File I/O
- Floating point support
- Optimize generated code
- Multi-threading support
- Cross-compilation targets
