# Stack Operations

Quarter has two stacks: data stack and return stack.

## Data Stack

Memory-based implementation with stack pointer.
- Located at: `0x000000-0x00FFFF` (64KB, 8K cells)
- Cell size: 8 bytes (64-bit)
- Grows upward

### Basic Operations

- `DUP` ( n -- n n ) - Duplicate top
- `DROP` ( n -- ) - Discard top
- `SWAP` ( a b -- b a ) - Exchange top two
- `OVER` ( a b -- a b a ) - Copy second
- `ROT` ( a b c -- b c a ) - Rotate top three
- `PICK` ( xu ... x0 u -- xu ... x0 xu ) - Copy u-th item
- `DEPTH` ( -- n ) - Number of items on stack

### Extended Operations (stdlib/core.fth)

- `NIP` ( a b -- b ) - Drop second
- `TUCK` ( a b -- b a b ) - Copy top below second
- `-ROT` ( a b c -- c a b ) - Rotate backwards
- `2DUP` ( a b -- a b a b ) - Duplicate top two
- `2DROP` ( a b -- ) - Drop top two
- `2SWAP` ( a b c d -- c d a b ) - Swap top two pairs
- `2OVER` ( a b c d -- a b c d a b ) - Copy second pair

### Stack Pointer Access

- `SP@` ( -- addr ) - Fetch data stack pointer
- `SP!` ( addr -- ) - Store data stack pointer

## Return Stack

Memory-based implementation with return stack pointer.
- Located at: `0x010000-0x01FFFF` (64KB, 8K cells)
- Used for temporary storage and call/return

### Operations

- `>R` ( n -- ) (R: -- n) - Move to return stack
- `R>` ( -- n ) (R: n -- ) - Move from return stack
- `R@` ( -- n ) (R: n -- n) - Copy from return stack (non-destructive)

### Return Stack Access

- `RP@` ( -- addr ) - Fetch return stack pointer
- `RP!` ( addr -- ) - Store return stack pointer

### Examples

```forth
: SAVE-AND-CALC ( a b c -- result )
  >R          \ Save c temporarily
  *           \ a * b
  R>          \ Retrieve c
  + ;         \ Add to product

3 4 5 SAVE-AND-CALC  \ → 17 (3*4 + 5)
```

## Implementation

- Data stack: `src/stack.rs`
- Return stack: `src/lib.rs` (ReturnStack)
- Both use Memory for storage
- Stack pointer operations enable pure Forth stack manipulation

## Safety Notes

- Return stack must be balanced (LIFO)
- Each `>R` must have matching `R>` within same word
- Improper use can crash interpreter
