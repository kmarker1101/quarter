# Error Handling

Quarter provides Forth-standard error handling words.

## Abort Words

### ABORT ( i*x -- ) ( R: j*x -- )
Clear both data and return stacks, then abort execution.

```forth
: DIVIDE-SAFE ( a b -- result )
  DUP 0= IF ABORT THEN
  / ;
```

**Behavior:**
- Clears data stack
- Clears return stack
- Prints "ABORT" to stderr
- Exits with code -1

### ABORT" ( flag -- ) - Compile-only
Conditionally abort with custom message.

```forth
: DIVIDE-SAFE ( a b -- result )
  DUP 0= ABORT" Division by zero!"
  / ;
```

**Behavior:**
- If flag is 0 (false): Continue normally
- If flag is non-zero (true): Print message to stderr and exit with code -2

**Examples:**
```forth
: VALIDATE-POSITIVE ( n -- n )
  DUP 0<= ABORT" Value must be positive!" ;

: SAFE-ARRAY-ACCESS ( index limit -- value )
  OVER OVER <= ABORT" Index out of bounds!"
  DROP
  CELLS ARRAY + @ ;
```

## Exception Words

### THROW ( k*x n -- k*x | i*x n )
If n is non-zero, unwind to nearest CATCH with error code n.
If n is zero, do nothing.

```forth
: TEST-THROW
  5 THROW  \ Throws error code 5
;
```

### CATCH ( i*x xt -- j*x 0 | i*x n )
Execute xt. If THROW occurs, returns error code n.
Otherwise returns 0.

```forth
: SAFE-EXECUTE ( xt -- flag )
  CATCH
  DUP IF
    ." Error code: " . CR
    TRUE
  ELSE
    DROP FALSE
  THEN ;

' TEST-THROW SAFE-EXECUTE  \ Catches error, prints "Error code: 5"
```

## Testing Error Handling

**Note:** ABORT and ABORT" with true flags cannot be tested in the test framework as they exit the program. Manual verification tests are in `/tmp/test_abort*.fth`.

**Test cases for ABORT":**
```forth
\ This works - flag is false
: TEST-ABORT-FALSE 0 ABORT" This should not print" 42 ;
T{ TEST-ABORT-FALSE -> 42 }T

\ This cannot be tested - would exit
\ : TEST-ABORT-TRUE -1 ABORT" This exits!" 99 ;
```

## Implementation

- ABORT: `src/words.rs::abort_word()`
- ABORT": `src/ast.rs::AstNode::AbortQuote`
- THROW: `src/words.rs::throw_word()`
- CATCH: `src/words.rs::catch_word()`

All error words print to stderr and use appropriate exit codes.
