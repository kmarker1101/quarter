# I/O Operations

Quarter supports character-level I/O and string literals.

## Character I/O

### EMIT ( c -- )
Output a single character by its Unicode code point.

```forth
65 EMIT           \ Prints: A
72 EMIT 105 EMIT 33 EMIT CR  \ Prints: Hi!
128515 EMIT CR    \ Prints: 😃 (Unicode smiley)
```

### KEY ( -- c )
Read a single character from input, push its code.

```forth
KEY .             \ Read char, print its code
KEY EMIT          \ Echo character
```

**Note:** In REPL, KEY is line-buffered (waits for Enter).

### SPACE ( -- )
Output a single space character (convenience for `32 EMIT`).

## Numeric Output

### . ( n -- )
Pop and print top of stack as signed decimal.

### U. ( u -- )
Pop and print top of stack as unsigned decimal.

### .R ( n width -- )
Print n right-aligned in field of given width.

### U.R ( u width -- )
Print u (unsigned) right-aligned in field of given width.

### .S ( -- )
Non-destructively print entire stack contents.

### CR ( -- )
Output a newline.

## String Literals

### ." ( "text" -- ) - Compile-only
Print string at runtime.

```forth
: GREET  ." Hello, World!" CR ;
GREET    \ Prints: Hello, World!
```

### S" ( "text" -- addr len )
Create string in memory, push address and length.

```forth
S" Hello"  \ → addr len
```

Works at top-level and in definitions. Stores string bytes in user memory.

### TYPE ( addr len -- )
Output string from memory.

```forth
S" Hello, World!" TYPE CR
```

## Implementation

- Character I/O: `src/words.rs` (EMIT, KEY, SPACE, TYPE)
- String literals: `src/lib.rs` parser creates PrintString/StackString nodes
- Output primitives: `src/words.rs` (dot, u_dot, dot_r, u_dot_r, dot_s, cr)
