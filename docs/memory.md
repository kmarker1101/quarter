# Memory Operations

Quarter provides 8MB byte-addressable memory with 64-bit cells.

## Memory Layout

```
0x000000-0x00FFFF  Data Stack    (64KB, 8K cells)
0x010000-0x01FFFF  Return Stack  (64KB, 8K cells)
0x020000-0x7FFFFF  User Memory   (~7.5MB)
```

## Memory Access

### ! ( n addr -- ) - Store cell (8 bytes)
### @ ( addr -- n ) - Fetch cell (8 bytes)
### C! ( c addr -- ) - Store byte
### C@ ( addr -- c ) - Fetch byte
### +! ( n addr -- ) - Add to memory location

## Memory Allocation

### HERE ( -- addr ) - Current dictionary pointer
### ALLOT ( n -- ) - Allocate n bytes
### , ( n -- ) - Compile cell to dictionary

### VARIABLE ( "name" -- )
Create a variable.

### CONSTANT ( n "name" -- )
Create a constant.

### CREATE ( "name" -- )
Create a data structure.

## Memory Alignment

### ALIGNED ( addr -- a-addr )
Round address up to 8-byte boundary.

```forth
0 ALIGNED   \ 0
1 ALIGNED   \ 8
9 ALIGNED   \ 16
```

### ALIGN ( -- )
Advance HERE to aligned boundary.

```forth
1 ALLOT    \ Make HERE unaligned
ALIGN      \ Round up to next 8-byte boundary
```

### FILL ( c-addr u char -- )
Fill memory region with byte value.

```forth
\ Fill 100 bytes at 500000 with zero
500000 100 0 FILL
```

## Implementation

All memory words defined in:
- `src/words.rs` (primitives)
- `stdlib/core.fth` (ALIGNED, ALIGN, FILL)
