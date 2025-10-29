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

\ OVER and ROT are now inline LLVM primitives
: NIP ( n1 n2 -- n2 ) SWAP DROP ;
: TUCK ( n1 n2 -- n2 n1 n2 ) SWAP OVER ;

\ Double-cell stack operations
: -ROT ( x1 x2 x3 -- x3 x1 x2 ) ROT ROT ;
: 2DUP ( a b -- a b a b ) OVER OVER ;
: 2DROP ( a b -- ) DROP DROP ;
: 2SWAP ( a b c d -- c d a b ) ROT >R ROT R> ;
: 2OVER ( a b c d -- a b c d a b ) >R >R 2DUP R> R> 2SWAP ;
: 2>R ( x1 x2 -- ) ( R: -- x1 x2 ) SWAP >R >R ;
: 2R> ( -- x1 x2 ) ( R: x1 x2 -- ) R> R> SWAP ;
: 2R@ ( -- x1 x2 ) ( R: x1 x2 -- x1 x2 ) R> R> 2DUP >R >R SWAP ;
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

\ SPACE is now an inline LLVM primitive

: SPACES ( n -- )
    DUP 0 > IF
      0 DO SPACE LOOP
    ELSE
      DROP
    THEN ;

\ Numeric base helpers
: DECIMAL ( -- )
    10 BASE ! ;

: HEX ( -- )
    16 BASE ! ;

: BINARY ( -- )
    2 BASE ! ;

\ WITHIN word for range checking
\ ( n lo hi -- flag ) Tests if lo <= n < hi (half-open interval)
: WITHIN ( n lo hi -- flag )
    OVER - >R - R> U< ;

\ =============================================================================
\ METAPROGRAMMING
\ =============================================================================

\ COUNT ( c-addr -- addr u )
\ Convert counted string to address/length pair
\ A counted string has its length in the first byte
: COUNT DUP 1+ SWAP C@ ;

\ =============================================================================
\ MEMORY ALIGNMENT
\ =============================================================================

\ ALIGNED ( addr -- a-addr )
\ Round up address to cell boundary
: ALIGNED DUP 7 AND IF 8 + -8 AND THEN ;

\ ALIGN ( -- )
\ Advance HERE to aligned boundary
: ALIGN HERE ALIGNED HERE - ALLOT ;

\ =============================================================================
\ MEMORY FILL
\ =============================================================================

\ FILL ( c-addr u char -- )
\ Fill memory region with character
: FILL -ROT 0 DO 2DUP I + C! LOOP 2DROP ;

\ =============================================================================
\ STRING MANIPULATION
\ =============================================================================

\ BLANK ( c-addr u -- )
\ Fill memory with spaces (BL = 32)
: BLANK ( c-addr u -- ) BL FILL ;

\ ERASE ( addr u -- )
\ Fill memory with zeros
: ERASE ( addr u -- ) 0 FILL ;

\ /STRING ( c-addr1 u1 n -- c-addr2 u2 )
\ Adjust string by advancing address and decreasing length
: /STRING ( c-addr1 u1 n -- c-addr2 u2 )
    OVER MIN       ( c-addr1 u1 n' )  \ clamp n to u1
    ROT OVER +     ( u1 n' c-addr2 )
    -ROT -         ( c-addr2 u2 )
;

\ COMPARE ( c-addr1 u1 c-addr2 u2 -- n )
\ Compare two strings byte-by-byte, return -1 (less), 0 (equal), or 1 (greater)
\ Now implemented as a primitive in words.rs

\ -TRAILING ( c-addr u1 -- c-addr u2 )
\ Remove trailing spaces from string
\ Now implemented as a primitive in words.rs

\ SEARCH ( c-addr1 u1 c-addr2 u2 -- c-addr3 u3 flag )
\ Search for substring c-addr2/u2 in string c-addr1/u1
\ Returns position where found (true flag) or original string (false flag)
\ Now implemented as a primitive in words.rs
