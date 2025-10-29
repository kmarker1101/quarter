# Word Reference

Complete reference of all implemented Forth words in Quarter.

## Table of Contents

- [Stack Operations](#stack-operations)
- [Arithmetic](#arithmetic)
- [Comparison](#comparison)
- [Bitwise Operations](#bitwise-operations)
- [Memory Access](#memory-access)
- [Memory Allocation](#memory-allocation)
- [Control Flow](#control-flow)
- [Character I/O](#character-io)
- [String Literals](#string-literals)
- [String Operations](#string-operations)
- [Numeric Output](#numeric-output)
- [Metaprogramming](#metaprogramming)
- [Error Handling](#error-handling)
- [File Operations](#file-operations)
- [Constants](#constants)

---

## Stack Operations

### Data Stack

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `DUP` | `( n -- n n )` | Duplicate top |
| `DROP` | `( n -- )` | Discard top |
| `SWAP` | `( a b -- b a )` | Exchange top two |
| `OVER` | `( a b -- a b a )` | Copy second |
| `ROT` | `( a b c -- b c a )` | Rotate top three |
| `PICK` | `( xu ... x0 u -- xu ... x0 xu )` | Copy u-th item |
| `DEPTH` | `( -- n )` | Number of items on stack |
| `NIP` | `( a b -- b )` | Drop second |
| `TUCK` | `( a b -- b a b )` | Copy top below second |
| `-ROT` | `( a b c -- c a b )` | Rotate backwards |
| `2DUP` | `( a b -- a b a b )` | Duplicate top two |
| `2DROP` | `( a b -- )` | Drop top two |
| `2SWAP` | `( a b c d -- c d a b )` | Swap top two pairs |
| `2OVER` | `( a b c d -- a b c d a b )` | Copy second pair |
| `SP@` | `( -- addr )` | Fetch data stack pointer |
| `SP!` | `( addr -- )` | Store data stack pointer |
| `.S` | `( -- )` | Non-destructively print stack |

### Return Stack

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `>R` | `( n -- ) (R: -- n)` | Move to return stack |
| `R>` | `( -- n ) (R: n -- )` | Move from return stack |
| `R@` | `( -- n ) (R: n -- n)` | Copy from return stack |
| `RP@` | `( -- addr )` | Fetch return stack pointer |
| `RP!` | `( addr -- )` | Store return stack pointer |

---

## Arithmetic

### Basic Operations

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `+` | `( a b -- sum )` | Addition |
| `-` | `( a b -- difference )` | Subtraction |
| `*` | `( a b -- product )` | Multiplication |
| `/` | `( a b -- quotient )` | Division (truncated) |
| `MOD` | `( a b -- remainder )` | Modulo |
| `/MOD` | `( a b -- remainder quotient )` | Combined division and modulo |
| `*/` | `( a b c -- result )` | Multiply then divide: (a*b)/c |

### Unary Operations

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `NEGATE` | `( n -- -n )` | Change sign |
| `ABS` | `( n -- \|n\| )` | Absolute value |
| `1+` | `( n -- n+1 )` | Increment by 1 |
| `1-` | `( n -- n-1 )` | Decrement by 1 |
| `2*` | `( n -- 2n )` | Multiply by 2 |
| `2/` | `( n -- n/2 )` | Divide by 2 |
| `2+` | `( n -- n+2 )` | Add 2 |
| `3+` to `11+` | `( n -- n+k )` | Add k (stdlib) |

### Min/Max

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `MIN` | `( a b -- min )` | Minimum of two |
| `MAX` | `( a b -- max )` | Maximum of two |

### Cell Arithmetic

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `CELLS` | `( n -- bytes )` | Convert cells to bytes (n * 8) |
| `CELL+` | `( addr -- addr+8 )` | Add one cell size |

---

## Comparison

All comparisons return -1 for true, 0 for false.

### Standard Comparisons

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `<` | `( a b -- flag )` | Less than |
| `>` | `( a b -- flag )` | Greater than |
| `=` | `( a b -- flag )` | Equal |
| `<>` | `( a b -- flag )` | Not equal |
| `<=` | `( a b -- flag )` | Less than or equal |
| `>=` | `( a b -- flag )` | Greater than or equal |
| `U<` | `( u1 u2 -- flag )` | Unsigned less than |

### Zero Comparisons

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `0=` | `( n -- flag )` | Equal to zero |
| `0<` | `( n -- flag )` | Less than zero |
| `0>` | `( n -- flag )` | Greater than zero |

---

## Bitwise Operations

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `AND` | `( n1 n2 -- n3 )` | Bitwise AND |
| `OR` | `( n1 n2 -- n3 )` | Bitwise OR |
| `XOR` | `( n1 n2 -- n3 )` | Bitwise XOR |
| `INVERT` | `( n -- ~n )` | Bitwise NOT |
| `LSHIFT` | `( n1 u -- n2 )` | Left shift by u bits |
| `RSHIFT` | `( n1 u -- n2 )` | Right shift by u bits |

---

## Memory Access

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `!` | `( n addr -- )` | Store cell (8 bytes) |
| `@` | `( addr -- n )` | Fetch cell (8 bytes) |
| `C!` | `( c addr -- )` | Store byte |
| `C@` | `( addr -- c )` | Fetch byte |
| `+!` | `( n addr -- )` | Add to memory location |

---

## Memory Allocation

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `HERE` | `( -- addr )` | Current dictionary pointer |
| `ALLOT` | `( n -- )` | Allocate n bytes |
| `,` | `( n -- )` | Compile cell to dictionary |
| `VARIABLE` | `( "name" -- )` | Create a variable |
| `CONSTANT` | `( n "name" -- )` | Create a constant |
| `CREATE` | `( "name" -- )` | Create a data structure |
| `ALIGNED` | `( addr -- a-addr )` | Round up to 8-byte boundary |
| `ALIGN` | `( -- )` | Advance HERE to aligned boundary |
| `FILL` | `( c-addr u char -- )` | Fill memory with byte value |
| `ERASE` | `( addr u -- )` | Fill memory with zeros |

---

## Control Flow

All control flow words are **compile-only** (must be used inside `:` `;`).

### Conditionals

| Word | Description |
|------|-------------|
| `IF` ... `THEN` | Conditional execution |
| `IF` ... `ELSE` ... `THEN` | Two-way conditional |

### Loops

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `BEGIN` ... `UNTIL` | `( -- )` loop: `( -- flag )` | Post-test loop |
| `BEGIN` ... `WHILE` ... `REPEAT` | `( -- )` loop: `( -- flag )` | Pre-test loop |
| `DO` ... `LOOP` | `( limit start -- )` | Counted loop (increment by 1) |
| `DO` ... `+LOOP` | `( limit start -- )` loop: `( n -- )` | Counted loop (variable increment) |
| `I` | `( -- n )` | Current loop index |
| `J` | `( -- n )` | Outer loop index |
| `LEAVE` | `( -- )` | Exit loop early |

### Word Control

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `EXIT` | `( -- )` | Exit word early |
| `RECURSE` | `( -- )` | Call current word recursively |

---

## Character I/O

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `EMIT` | `( c -- )` | Output character by code point |
| `KEY` | `( -- c )` | Read character, return code |
| `SPACE` | `( -- )` | Output a space |
| `CR` | `( -- )` | Output a newline |
| `TYPE` | `( addr len -- )` | Output string from memory |

---

## String Literals

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `S"` _text_`"` | `( -- addr len )` | Create string, push addr/length |
| `C"` _text_`"` | `( -- c-addr )` | Create null-terminated string |
| `."` _text_`"` | `( -- )` | Print string (compile-only) |

---

## String Operations

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `COMPARE` | `( c-addr1 u1 c-addr2 u2 -- n )` | Compare strings (-1/0/1) |
| `-TRAILING` | `( c-addr u1 -- c-addr u2 )` | Remove trailing spaces |
| `/STRING` | `( c-addr1 u1 n -- c-addr2 u2 )` | Adjust string pointer/length |
| `SEARCH` | `( c-addr1 u1 c-addr2 u2 -- c-addr3 u3 flag )` | Search for substring |
| `COUNT` | `( c-addr -- addr u )` | Convert counted string to addr/len |

---

## Numeric Output

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `.` | `( n -- )` | Print signed decimal |
| `U.` | `( u -- )` | Print unsigned decimal |
| `.R` | `( n width -- )` | Print right-aligned in field |
| `U.R` | `( u width -- )` | Print unsigned right-aligned |

---

## Metaprogramming

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `EXECUTE` | `( xt -- )` | Execute word from execution token |
| `'` (TICK) | `( "name" -- xt )` | Get execution token at runtime |
| `[']` | `( "name" -- xt )` | Get execution token (compile-only) |
| `CHAR` | `( "name" -- char )` | Get ASCII of first character |
| `[CHAR]` | `( "name" -- char )` | Get ASCII (compile-only) |
| `FIND` | `( c-addr -- c-addr 0 \| xt 1 \| xt -1 )` | Search dictionary |
| `IMMEDIATE` | `( -- )` | Mark last word as immediate |
| `>NUMBER` | `( ud1-lo ud1-hi c-addr u -- ud2-lo ud2-hi c-addr' u' )` | Convert string to number |

---

## Error Handling

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `ABORT` | `( i*x -- ) (R: j*x -- )` | Clear stacks and abort |
| `ABORT"` _msg_`"` | `( flag -- )` | Conditional abort with message (compile-only) |
| `THROW` | `( k*x n -- k*x \| i*x n )` | Throw exception if n ≠ 0 |
| `CATCH` | `( i*x xt -- j*x 0 \| i*x n )` | Catch exceptions from xt |

---

## File Operations

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `INCLUDE` | `( "name" -- )` | Load and execute file |
| `INCLUDED` | `( c-addr u -- )` | Load file by addr/len |

---

## Constants

| Word | Stack Effect | Value | Description |
|------|--------------|-------|-------------|
| `TRUE` | `( -- -1 )` | -1 | Boolean true |
| `FALSE` | `( -- 0 )` | 0 | Boolean false |
| `BL` | `( -- 32 )` | 32 | Space character |

---

## Word Definition

| Word | Stack Effect | Description |
|------|--------------|-------------|
| `:` _name_ | `( -- )` | Begin word definition |
| `;` | `( -- )` | End word definition |

---

## Implementation Notes

- **Cell size**: 8 bytes (64-bit integers)
- **Case-insensitive**: All words converted to uppercase
- **Boolean values**: -1 for true, 0 for false
- **Memory**: 8MB total (see [memory.md](memory.md) for layout)

## See Also

- **[JIT Compilation](jit-compilation.md)** - Runtime performance
- **[AOT Compilation](aot-compilation.md)** - Standalone executables
- **[Control Flow](control-flow.md)** - Detailed control flow examples
- **[Memory](memory.md)** - Memory layout and operations
- **[Strings](strings.md)** - String manipulation details
- **[Metaprogramming](metaprogramming.md)** - Execution tokens and FIND
- **[Error Handling](error-handling.md)** - Exception handling
- **[I/O](io.md)** - Character and string I/O
- **[Arithmetic](arithmetic.md)** - Math operations
