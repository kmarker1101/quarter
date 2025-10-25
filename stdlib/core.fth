\ Quarter Forth Standard Library
\ Core definitions for the Forth interpreter

\ =============================================================================
\ CONSTANTS
\ =============================================================================

0 CONSTANT FALSE
-1 CONSTANT TRUE
32 CONSTANT BL

\ =============================================================================
\ COMPARISON OPERATORS
\ =============================================================================

\ All comparison operators are now inline LLVM primitives
\ <, >, =, <>, <=, >=, 0=, 0<, 0> compile directly to LLVM icmp + sext instructions

\ =============================================================================
\ STACK MANIPULATION
\ =============================================================================

: OVER ( x1 x2 -- x1 x2 x1 ) >R DUP R> SWAP ;
: NIP ( n1 n2 -- n2 ) SWAP DROP ;
: TUCK ( n1 n2 -- n2 n1 n2 ) SWAP OVER ;

\ Double-cell stack operations
: ROT ( x1 x2 x3 -- x2 x3 x1 ) >R SWAP R> SWAP ;
: -ROT ( x1 x2 x3 -- x3 x1 x2 ) ROT ROT ;
: 2DUP ( a b -- a b a b ) OVER OVER ;
: 2DROP ( a b -- ) DROP DROP ;
: 2SWAP ( a b c d -- c d a b ) ROT >R ROT R> ;
: 2OVER ( a b c d -- a b c d a b ) >R >R 2DUP R> R> 2SWAP ;
\ =============================================================================
\ ARITHMETIC
\ =============================================================================

\ NEGATE, ABS, MIN, MAX, 1+, 1-, 2*, 2/ are now primitives with JIT wrappers
: CELLS ( n -- n ) 8 * ;
: CELL+ ( a-addr1 -- a-addr2 ) 8 + ;
: +! ( n addr -- ) DUP @ ROT + SWAP ! ;
: 2+ ( n -- n+2 ) 2 + ;
: 3+ ( n -- n+3 ) 3 + ;
: 4+ ( n -- n+4 ) 4 + ;
: 5+ ( n -- n+5 ) 5 + ;
: 6+ ( n -- n+6 ) 6 + ;
: 7+ ( n -- n+7 ) 7 + ;
: 8+ ( n -- n+8 ) 8 + ;
: 9+ ( n -- n+9 ) 9 + ;
: 10+ ( n -- n+10 ) 10 + ;
: 11+ ( n -- n+11 ) 11 + ;

\ MOD is now an inline LLVM primitive (srem instruction)

\ =============================================================================
\ INPUT/OUTPUT
\ =============================================================================

: SPACE BL EMIT ;

: SPACES ( n -- )
    DUP 0 > IF
      0 DO SPACE LOOP
    ELSE
      DROP
    THEN ;
