# Quarter Documentation

Detailed documentation for Quarter Forth features and implementation.

## Contents

### User Documentation

- **[AOT Compilation](aot-compilation.md)** - Compiling Forth to standalone executables
- **[Control Flow](control-flow.md)** - IF/THEN/ELSE, loops, LEAVE, EXIT, RECURSE
- **[Memory Operations](memory.md)** - Memory access, allocation, alignment, FILL
- **[Metaprogramming](metaprogramming.md)** - EXECUTE, TICK, FIND, IMMEDIATE
- **[I/O Operations](io.md)** - Character I/O, strings, output
- **[String Operations](strings.md)** - String manipulation, comparison, search
- **[Stack Operations](stacks.md)** - Data stack and return stack
- **[Error Handling](error-handling.md)** - ABORT, ABORT", CATCH, THROW
- **[Arithmetic](arithmetic.md)** - Math operations and comparisons

### Developer Documentation

- **[Adding LLVM Primitives](adding-llvm-primitives.md)** - Step-by-step guide for implementing new primitives
- **[LLVM Global String Implementation](llvm-global-strings-notes.md)** - Dual-strategy string handling for JIT and AOT modes

## Quick Start

See the main [CLAUDE.md](../CLAUDE.md) for build commands and quick reference.

Each documentation file includes:
- Word descriptions with stack effects
- Usage examples
- Implementation notes
- Common patterns and idioms
