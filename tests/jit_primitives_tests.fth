\ Tests for JIT-compiled primitives
\ These primitives have both Rust implementations and JIT wrappers
\
\ Run in interpreter mode: ./quarter tests/jit_primitives_tests.fth
\ Run with JIT compiler:   ./quarter --forth-compiler tests/jit_primitives_tests.fth
\
\ All tests should produce identical output in both modes.

\ =============================================================================
\ EQUALS COMPARISON (=)
\ =============================================================================

: TEST-EQUAL-SAME 5 5 = . CR ;
: TEST-EQUAL-DIFF 5 3 = . CR ;
: TEST-EQUAL-ZERO 0 0 = . CR ;
: TEST-EQUAL-NEG -5 -5 = . CR ;
: TEST-EQUAL-MIXED -5 5 = . CR ;
: TEST-EQUAL-LARGE 1000000 1000000 = . CR ;

\ =============================================================================
\ STACK POINTER PRIMITIVES (SP@, SP!)
\ =============================================================================

: TEST-SP-FETCH SP@ DROP 1 . CR ;

\ =============================================================================
\ RETURN STACK POINTER PRIMITIVES (RP@, RP!)
\ =============================================================================

: TEST-RP-FETCH RP@ DROP 1 . CR ;

\ =============================================================================
\ MEMORY ALLOCATION PRIMITIVES (HERE, ALLOT, comma)
\ =============================================================================

: TEST-HERE-VALID HERE 131072 >= . CR ;
: TEST-ALLOT-16 HERE 16 ALLOT HERE SWAP - . CR ;
: TEST-COMMA-ADVANCE HERE 42 , HERE SWAP - . CR ;
: TEST-COMMA-STORE-FETCH HERE DUP 777 , @ . CR ;

\ =============================================================================
\ I/O FORMATTING PRIMITIVES (U., .R, U.R)
\ =============================================================================

: TEST-U-DOT 42 U. CR ;
: TEST-U-DOT-NEG -1 U. CR ;
: TEST-DOT-R-5 42 5 .R CR ;
: TEST-DOT-R-3 100 3 .R CR ;
: TEST-U-DOT-R-6 99 6 U.R CR ;

\ =============================================================================
\ TYPE PRIMITIVE (string output)
\ =============================================================================

: TEST-TYPE-HI
    72  200000 C!   \ H
    105 200001 C!   \ i
    33  200002 C!   \ !
    200000 3 TYPE CR
;

: TEST-TYPE-HELLO
    72  200010 C!   \ H
    101 200011 C!   \ e
    108 200012 C!   \ l
    108 200013 C!   \ l
    111 200014 C!   \ o
    200010 5 TYPE CR
;

\ =============================================================================
\ RUN ALL TESTS
\ =============================================================================

\ Equals tests (should print: -1, 0, -1, -1, 0, -1)
TEST-EQUAL-SAME
TEST-EQUAL-DIFF
TEST-EQUAL-ZERO
TEST-EQUAL-NEG
TEST-EQUAL-MIXED
TEST-EQUAL-LARGE

\ Stack pointer tests (should print: 1)
TEST-SP-FETCH

\ Return stack pointer tests (should print: 1)
TEST-RP-FETCH

\ Memory allocation tests (should print: -1, 16, 8, 777)
TEST-HERE-VALID
TEST-ALLOT-16
TEST-COMMA-ADVANCE
TEST-COMMA-STORE-FETCH

\ I/O formatting tests
TEST-U-DOT          \ 42
TEST-U-DOT-NEG      \ 18446744073709551615 (max u64)
TEST-DOT-R-5        \    42
TEST-DOT-R-3        \ 100
TEST-U-DOT-R-6      \     99

\ TYPE tests
TEST-TYPE-HI        \ Hi!
TEST-TYPE-HELLO     \ Hello
