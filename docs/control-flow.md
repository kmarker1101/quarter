# Control Flow

Quarter supports standard Forth control flow structures.

## Conditional Execution

### IF/THEN/ELSE ( flag -- )
```forth
: ABS ( n -- |n| )
  DUP 0< IF NEGATE THEN ;

: MAX ( a b -- max )
  2DUP < IF SWAP THEN DROP ;
```

## Loops

### BEGIN...UNTIL ( -- ) (loop: -- flag )
Post-test loop - executes body at least once.

### BEGIN...WHILE...REPEAT ( -- ) (loop: -- flag )
Pre-test loop - tests condition before each iteration.

### DO...LOOP ( limit start -- )
Counted loop with fixed increment.

### +LOOP ( limit start -- ) (loop: increment -- )
Counted loop with variable increment.

## Loop Control

- `I` - Current loop index
- `J` - Outer loop index (for nested loops)
- `LEAVE` - Early loop exit
- `EXIT` - Early word return

## Recursion

### RECURSE
Self-recursion word that calls the current definition.

```forth
: FACTORIAL ( n -- n! )
  DUP 1 <= IF DROP 1 ELSE DUP 1 - RECURSE * THEN ;
```

Works in both interpreted and JIT modes.

## Implementation Notes

- All control flow words are **compile-only** (must be used inside `:` `;`)
- IF/THEN/ELSE compiled to AstNode::IfThenElse
- Loops maintain LoopStack for tracking indices
- RECURSE enables tail-call optimization
