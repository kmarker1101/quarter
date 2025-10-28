# Arithmetic and Comparison

Quarter provides standard Forth arithmetic and comparison operations.

## Arithmetic Operations

### Basic Operations
- `+` ( a b -- sum ) - Addition
- `-` ( a b -- difference ) - Subtraction
- `*` ( a b -- product ) - Multiplication
- `/` ( a b -- quotient ) - Division (truncated)
- `MOD` ( a b -- remainder ) - Modulo
- `/MOD` ( a b -- remainder quotient ) - Combined division and modulo

### Extended Operations
- `*/` ( a b c -- result ) - Multiply then divide: (a*b)/c
- `NEGATE` ( n -- -n ) - Change sign
- `ABS` ( n -- |n| ) - Absolute value
- `MIN` ( a b -- min ) - Minimum of two numbers
- `MAX` ( a b -- max ) - Maximum of two numbers

### Increment/Decrement
- `1+` ( n -- n+1 ) - Increment by 1
- `1-` ( n -- n-1 ) - Decrement by 1
- `2*` ( n -- 2n ) - Multiply by 2 (left shift)
- `2/` ( n -- n/2 ) - Divide by 2 (right shift)

From stdlib/core.fth: `2+`, `3+`, ... `11+`

## Comparison Operations

All comparisons return -1 for true, 0 for false.

### Standard Comparisons
- `<` ( a b -- flag ) - Less than
- `>` ( a b -- flag ) - Greater than
- `=` ( a b -- flag ) - Equal
- `<>` ( a b -- flag ) - Not equal
- `<=` ( a b -- flag ) - Less than or equal
- `>=` ( a b -- flag ) - Greater than or equal

### Unsigned Comparison
- `U<` ( u1 u2 -- flag ) - Unsigned less than

### Zero Comparisons
- `0=` ( n -- flag ) - Equal to zero
- `0<` ( n -- flag ) - Less than zero (negative)
- `0>` ( n -- flag ) - Greater than zero (positive)

**Why zero comparisons?**
- More efficient (single operation)
- More readable
- Common in Forth idioms

```forth
: ABS ( n -- |n| )
  DUP 0< IF NEGATE THEN ;

: SIGN ( n -- -1|0|1 )
  DUP 0< IF DROP -1 EXIT THEN
  DUP 0> IF DROP 1 EXIT THEN
  DROP 0 ;
```

## Bitwise Operations

- `AND` ( n1 n2 -- n3 ) - Bitwise AND
- `OR` ( n1 n2 -- n3 ) - Bitwise OR
- `XOR` ( n1 n2 -- n3 ) - Bitwise XOR
- `INVERT` ( n -- ~n ) - Bitwise NOT
- `LSHIFT` ( n1 u -- n2 ) - Left shift by u bits
- `RSHIFT` ( n1 u -- n2 ) - Right shift by u bits (arithmetic)

**Boolean logic:** Forth uses -1 for true, 0 for false:
```forth
-1 -1 AND .   \ true AND true = -1 (true)
-1 0 AND .    \ true AND false = 0 (false)
```

## Cell Arithmetic

- `CELLS` ( n -- bytes ) - Convert cell count to bytes (n * 8)
- `CELL+` ( addr -- addr+8 ) - Add one cell size

Defined in `stdlib/core.fth`:
```forth
: CELLS 8 * ;
: CELL+ 8 + ;
```

## Constants

- `TRUE` ( -- -1 ) - Boolean true
- `FALSE` ( -- 0 ) - Boolean false
- `BL` ( -- 32 ) - Space character

Defined in `stdlib/core.fth`:
```forth
0 CONSTANT FALSE
-1 CONSTANT TRUE
32 CONSTANT BL
```

## Implementation

- Basic arithmetic: `src/words.rs` (primitives)
- Comparisons: `src/words.rs` (primitives) + `stdlib/core.fth`
- Bitwise: `src/words.rs` (primitives)
- Extended operations: `stdlib/core.fth`

All arithmetic uses 64-bit signed integers (i64).
