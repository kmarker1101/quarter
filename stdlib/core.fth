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

: 0= ( n -- flag ) IF FALSE ELSE TRUE THEN ;
: = ( n1 n2 -- flag ) - 0= ;
: 0< ( n -- flag ) 0 < ;
: 0> ( n -- flag ) 0 > ;
: <> ( n1 n2 -- flag ) = 0= ;
: <= ( n1 n2 -- flag ) > 0= ;
: >= ( n1 n2 -- flag ) < 0= ;

\ =============================================================================
\ STACK MANIPULATION
\ =============================================================================

: OVER ( x1 x2 -- x1 x2 x1 ) >R DUP R> SWAP ;
: NIP ( n1 n2 -- n2 ) SWAP DROP ;
: TUCK ( n1 n2 -- n2 n1 n2 ) SWAP OVER ;

\ Double-cell stack operations
: ROT ( x1 x2 x3 -- x2 x3 x1 ) >R SWAP R> SWAP ;
: 2DUP ( a b -- a b a b ) OVER OVER ;
: 2DROP ( a b -- ) DROP DROP ;
: 2SWAP ( a b c d -- c d a b ) ROT >R ROT R> ;
: 2OVER ( a b c d -- a b c d a b ) >R >R 2DUP R> R> 2SWAP ;
\ =============================================================================
\ ARITHMETIC
\ =============================================================================

: NEGATE ( n1 -- n2 ) 0 SWAP - ;
: ABS ( n -- +n ) DUP 0 < IF NEGATE THEN ;
: CELLS ( n -- n ) 4 * ;
: CELL+ ( a-addr1 -- a-addr2 ) 4 + ;
: +! ( n addr -- ) DUP @ ROT + SWAP ! ;
: 1+ ( n -- n+1 ) 1 + ;
: 1- ( n -- n-1 ) 1 - ;
: 2* ( n -- n*2 ) 2 * ;
: 2/ ( n -- n/2 ) 2 / ;

: MIN ( n1 n2 -- n ) 2DUP > IF SWAP THEN DROP ;
: MAX ( n1 n2 -- n ) 2DUP < IF SWAP THEN DROP ;

: MOD ( n1 n2 -- remainder ) /MOD DROP ;

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
