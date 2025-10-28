# Metaprogramming

Quarter supports powerful metaprogramming features for dynamic word execution and dictionary manipulation.

## Execution Tokens

An **execution token** (xt) is a memory address pointing to a counted string representation of a word name:
- Format: `[length byte][character bytes...]`
- Example: Word "DUP" → `[3]['D']['U']['P']`

## Core Words

### EXECUTE ( xt -- )
Execute a word from its execution token.

```forth
5 ' DUP EXECUTE  \ Get xt for DUP, execute it → 5 5
```

### ' (TICK) ( "name" -- xt )
Get execution token for a word at runtime.

```forth
' DUP      \ Returns xt for DUP
' SQUARE   \ Returns xt for user-defined SQUARE
```

Works both at top-level and in definitions.

### ['] ( "name" -- xt ) - Compile-only
Compile-time version of TICK. More efficient for use in definitions.

```forth
: TEST-DUP ['] DUP EXECUTE ;
10 TEST-DUP  \ → 10 10
```

### CHAR ( "name" -- char )
Get ASCII value of first character of next word.

```forth
CHAR A     \ → 65
CHAR HELLO \ → 72 (H)
```

### [CHAR] ( "name" -- char ) - Compile-only
Compile-time version of CHAR.

### COUNT ( c-addr -- addr u )
Convert counted string to address/length pair.

```forth
\ Given counted string at 500000: [2]['H']['I']
500000 COUNT  \ → 500001 2
```

Defined in `stdlib/core.fth`:
```forth
: COUNT DUP 1+ SWAP C@ ;
```

## Number Parsing

### >NUMBER ( ud1-lo ud1-hi c-addr u -- ud2-lo ud2-hi c-addr' u' )
Convert string to double-cell unsigned number with accumulation.

**Parameters:**
- `ud1-lo ud1-hi`: Initial double-cell accumulator (64-bit low, 64-bit high)
- `c-addr u`: String address and length
- `ud2-lo ud2-hi`: Updated accumulator
- `c-addr' u'`: Remaining unconverted portion

**Behavior:**
- Converts characters based on current BASE (2-36)
- Accumulates into 128-bit unsigned number
- Stops at first non-digit character
- Returns address and length of remaining unconverted string

```forth
\ Store "123" at address 600000
3 600000 C!
49 600001 C!  \ '1'
50 600002 C!  \ '2'
51 600003 C!  \ '3'

0 0 600001 3 >NUMBER  \ → 123 0 600004 0

\ Hex conversion
2 600100 C!
70 600101 C!  \ 'F'
70 600102 C!  \ 'F'
HEX
0 0 600101 2 >NUMBER  \ → 255 0 600103 0
DECIMAL

\ Stops at invalid character
\ "12X45" at 600200
0 0 600200 5 >NUMBER  \ → 12 0 600203 3 (converted "12", "X45" remains)

\ Accumulation example
100 0 600301 2 >NUMBER  \ Start with 100, add digits → 10099 0
```

## Dictionary Manipulation

### FIND ( c-addr -- c-addr 0 | xt 1 | xt -1 )
Search dictionary for word by counted string name.

**Returns:**
- If found and immediate: `( c-addr -- xt 1 )`
- If found and not immediate: `( c-addr -- xt -1 )`
- If not found: `( c-addr -- c-addr 0 )`

```forth
\ Create counted string "DUP" at 500000
3 500000 C!      \ Length
68 500001 C!     \ D
85 500002 C!     \ U
80 500003 C!     \ P

500000 FIND      \ Search for DUP
.                \ Prints: -1 (not immediate)
```

### IMMEDIATE ( -- )
Mark the most recently defined word as immediate.

**Usage pattern:**
```forth
: MY-WORD ... ;
IMMEDIATE
```

Immediate words execute during compilation instead of being compiled.

## Examples

**Dynamic word selection:**
```forth
: OPERATION ( n1 n2 op-xt -- result )
  EXECUTE ;

5 3 ' + OPERATION  \ → 8
5 3 ' * OPERATION  \ → 15
```

**Function table:**
```forth
CREATE OPS
  ' + , ' - , ' * , ' / ,

: NTH-OP ( n -- xt )
  CELLS OPS + @ ;

5 3 0 NTH-OP EXECUTE  \ → 8 (addition)
```

## Implementation

- Execution tokens created in `src/ast.rs` (TickLiteral node)
- FIND implemented as AstNode::Find
- IMMEDIATE tracked in Dictionary::immediate_words
