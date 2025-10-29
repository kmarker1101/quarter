# String Operations

Quarter provides ANS Forth-compatible string manipulation words.

## String Literals

### S" ( "ccc<quote>" -- c-addr u )
Compile: Parse string delimited by `"` and append to dictionary.
Execute: Push address and length of string.

```forth
S" Hello, World!" TYPE CR  \ Print string
S" test.txt" INCLUDED      \ Load file
```

**Modes:**
- **Interpreted:** Allocates string at HERE, pushes addr/length
- **JIT/AOT:** Compiles string constant, generates code to push addr/length
- Works in word definitions and at top level

### C" ( "ccc<quote>" -- c-addr )
Parse string and create null-terminated version.
Push address of null-terminated string.

```forth
C" filename.txt" OPEN-FILE  \ C-style string
```

**Implementation:** Stores string bytes + null terminator at HERE, pushes address.

### ." ( "ccc<quote>" -- )
**Compile-only.** Parse and print string at runtime.

```forth
: GREET  ." Hello, World!" CR ;
GREET  \ Prints: Hello, World!
```

## String Comparison

### COMPARE ( c-addr1 u1 c-addr2 u2 -- n )
Lexicographic string comparison.

**Returns:**
- `-1` if string1 < string2
- `0` if strings are equal
- `1` if string1 > string2

```forth
S" ABC" S" ABC" COMPARE .  \ 0 (equal)
S" ABC" S" ABD" COMPARE .  \ -1 (less)
S" ABD" S" ABC" COMPARE .  \ 1 (greater)
```

**Algorithm:**
1. Compare byte-by-byte up to minimum length
2. If bytes differ, return comparison result
3. If all bytes equal, compare lengths

**Implementation:** LLVM primitive in `src/runtime.rs`

## String Manipulation

### -TRAILING ( c-addr u1 -- c-addr u2 )
Remove trailing spaces from string.

```forth
S" ABC  " -TRAILING  \ addr 3 (removed 2 spaces)
S" ABC" -TRAILING    \ addr 3 (no change)
S"   " -TRAILING     \ addr 0 (all spaces removed)
```

**Returns:** Original address, adjusted length (u2 ≤ u1)

**Implementation:** LLVM primitive in `src/runtime.rs`

### /STRING ( c-addr1 u1 n -- c-addr2 u2 )
Adjust string pointer and length.

```forth
S" HELLO" 2 /STRING TYPE  \ Prints: LLO
\ Advances addr by 2, reduces length by 2
```

**Stack effect:**
- c-addr2 = c-addr1 + n
- u2 = u1 - n

**Implementation:** Forth in `stdlib/core.fth`

```forth
: /STRING ( c-addr1 u1 n -- c-addr2 u2 )
    OVER MIN       \ Clamp n to string length
    ROT OVER +     \ New address
    -ROT - ;       \ New length
```

### ERASE ( addr u -- )
Fill memory region with zeros.

```forth
500000 100 ERASE  \ Zero 100 bytes at address 500000
```

**Equivalent to:** `0 FILL`

**Implementation:** Forth in `stdlib/core.fth`

```forth
: ERASE ( addr u -- ) 0 FILL ;
```

## Substring Search

### SEARCH ( c-addr1 u1 c-addr2 u2 -- c-addr3 u3 flag )
Search for substring (c-addr2 u2) within string (c-addr1 u1).

**If found:**
- c-addr3 = address of match within haystack
- u3 = remaining length from match to end
- flag = TRUE (-1)

**If not found:**
- c-addr3 = original c-addr1
- u3 = original u1
- flag = FALSE (0)

```forth
S" HELLO WORLD" S" WORLD" SEARCH
\ Returns: addr-of-WORLD 5 -1

S" HELLO" S" XYZ" SEARCH
\ Returns: addr-of-HELLO 5 0
```

**Empty needle:** Always matches (returns TRUE)

**Algorithm:** Naive string search - checks each position for match.

**Implementation:** LLVM primitive in `src/runtime.rs`

## Practical Examples

### Extract substring

```forth
: SUBSTRING ( c-addr u start len -- c-addr' u' )
    >R OVER + R> ;

S" Hello, World!" 7 5 SUBSTRING TYPE  \ Prints: World
```

### Trim whitespace

```forth
: TRIM ( c-addr u -- c-addr' u' )
    \ Trim leading spaces
    BEGIN
        DUP 0> WHILE
        OVER C@ 32 = WHILE  \ Space?
        1 /STRING
    REPEAT THEN
    \ Trim trailing spaces
    -TRAILING ;

S"   Hello  " TRIM TYPE  \ Prints: Hello
```

### Case-insensitive search

```forth
: UPPER-CHAR ( c -- c' )
    DUP 97 >= OVER 122 <= AND IF
        32 -  \ Convert a-z to A-Z
    THEN ;

: STRING-UPPER ( c-addr u -- )
    0 DO
        DUP I + DUP C@
        UPPER-CHAR
        SWAP C!
    LOOP DROP ;
```

### Split string

```forth
: SPLIT-AT ( c-addr u n -- c-addr1 u1 c-addr2 u2 )
    2DUP >R >R  \ Save for second part
    MIN         \ First part: take n chars
    2DUP        \ Copy for second part start
    R> R>       \ Get original u and n
    ROT + -ROT  \ Calculate second part
    DROP SWAP ; \ Clean up
```

## Performance Notes

**COMPARE, -TRAILING, SEARCH** are LLVM primitives:
- **Interpreted mode:** Direct Rust implementation
- **JIT mode:** Compiled to native code
- **AOT mode:** Linked as external functions

**S" in JIT/AOT:**
- Strings allocated during compilation
- Address/length pushed as constants
- Zero runtime overhead for literal lookup

## Implementation Details

### File Locations

**Primitives:**
- `src/runtime.rs` - quarter_compare, quarter_minus_trailing, quarter_search
- `src/words.rs` - Extern declarations and wrappers
- `src/dictionary.rs` - Primitive registration
- `src/llvm_forth.rs` - Symbol registration for linking

**Compiler Support:**
- `stdlib/compiler.fth` - S" handler (AST type 9), -TRAILING name mapping
- `stdlib/core.fth` - ERASE, /STRING implementations

**Tests:**
- `tests/basic_tests.fth` - Comprehensive string word tests

### Architecture

String primitives follow Quarter's dual-implementation pattern:
1. **Runtime implementation** in `runtime.rs` (single source of truth)
2. **Extern declarations** in `words.rs` for linking
3. **Symbol registration** in `llvm_forth.rs` for AOT
4. **Primitive declarations** in `compiler.fth` for LLVM codegen

### S" Compiler Implementation

**AST Node:** `StackString(String)` (type 9)

**Compilation steps:**
1. Extract string from AST node
2. Allocate at HERE during compilation
3. Store bytes in dictionary
4. Advance HERE by string length
5. Generate IR to push address constant
6. Generate IR to push length constant

**Code location:** `stdlib/compiler.fth` lines 2618-2656

### Known Limitations

**AOT S" persistence:** Strings allocated during Forth compilation don't persist to the executable. Requires LLVM global string constant support (future enhancement).

**Workaround:** Use AOT for computational code, interpreted mode for string-heavy code, or pre-define strings as globals.

## See Also

- [I/O Operations](io.md) - TYPE, EMIT, character I/O
- [Memory Operations](memory.md) - Memory access and allocation
- [Control Flow](control-flow.md) - Loops for string processing
