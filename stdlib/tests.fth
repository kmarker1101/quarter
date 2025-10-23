\ Quarter Forth Standard Library Test Suite
\ Run with: RUN-ALL-TESTS

\ =============================================================================
\ COMPARISON TESTS
\ =============================================================================

: TEST-0=
    ." Testing 0=..." CR
    0 0= TRUE = IF ." 0 0= ok" ELSE ." 0 0= FAIL" THEN CR
    5 0= FALSE = IF ." 5 0= ok" ELSE ." 5 0= FAIL" THEN CR
    -3 0= FALSE = IF ." -3 0= ok" ELSE ." -3 0= FAIL" THEN CR ;

: TEST-=
    ." Testing =..." CR
    5 5 = TRUE = IF ." 5 5 = ok" ELSE ." 5 5 = FAIL" THEN CR
    5 3 = FALSE = IF ." 5 3 = ok" ELSE ." 5 3 = FAIL" THEN CR
    -2 -2 = TRUE = IF ." -2 -2 = ok" ELSE ." -2 -2 = FAIL" THEN CR ;

: TEST-0<
    ." Testing 0<..." CR
    -5 0< TRUE = IF ." -5 0< ok" ELSE ." -5 0< FAIL" THEN CR
    0 0< FALSE = IF ." 0 0< ok" ELSE ." 0 0< FAIL" THEN CR
    5 0< FALSE = IF ." 5 0< ok" ELSE ." 5 0< FAIL" THEN CR ;

: TEST-0>
    ." Testing 0>..." CR
    5 0> TRUE = IF ." 5 0> ok" ELSE ." 5 0> FAIL" THEN CR
    0 0> FALSE = IF ." 0 0> ok" ELSE ." 0 0> FAIL" THEN CR
    -5 0> FALSE = IF ." -5 0> ok" ELSE ." -5 0> FAIL" THEN CR ;

: TEST-<>
    ." Testing <>..." CR
    5 3 <> TRUE = IF ." 5 3 <> ok" ELSE ." 5 3 <> FAIL" THEN CR
    5 5 <> FALSE = IF ." 5 5 <> ok" ELSE ." 5 5 <> FAIL" THEN CR
    -2 3 <> TRUE = IF ." -2 3 <> ok" ELSE ." -2 3 <> FAIL" THEN CR ;

: TEST-<=
    ." Testing <=..." CR
    3 5 <= TRUE = IF ." 3 5 <= ok" ELSE ." 3 5 <= FAIL" THEN CR
    5 5 <= TRUE = IF ." 5 5 <= ok" ELSE ." 5 5 <= FAIL" THEN CR
    5 3 <= FALSE = IF ." 5 3 <= ok" ELSE ." 5 3 <= FAIL" THEN CR
    -5 0 <= TRUE = IF ." -5 0 <= ok" ELSE ." -5 0 <= FAIL" THEN CR ;

: TEST->=
    ." Testing >=..." CR
    5 3 >= TRUE = IF ." 5 3 >= ok" ELSE ." 5 3 >= FAIL" THEN CR
    5 5 >= TRUE = IF ." 5 5 >= ok" ELSE ." 5 5 >= FAIL" THEN CR
    3 5 >= FALSE = IF ." 3 5 >= ok" ELSE ." 3 5 >= FAIL" THEN CR
    0 -5 >= TRUE = IF ." 0 -5 >= ok" ELSE ." 0 -5 >= FAIL" THEN CR ;

\ =============================================================================
\ CONSTANT TESTS
\ =============================================================================

: TEST-CONSTANTS
    ." Testing constants..." CR
    TRUE -1 = IF ." TRUE ok" ELSE ." TRUE FAIL" THEN CR
    FALSE 0 = IF ." FALSE ok" ELSE ." FALSE FAIL" THEN CR
    BL 32 = IF ." BL ok" ELSE ." BL FAIL" THEN CR ;

\ =============================================================================
\ STACK MANIPULATION TESTS
\ =============================================================================

: TEST-OVER
    ." Testing OVER..." CR
    5 10 OVER
    5 = IF ." Top ok" ELSE ." Top FAIL" THEN CR
    10 = IF ." 2nd ok" ELSE ." 2nd FAIL" THEN CR
    5 = IF ." 3rd ok" ELSE ." 3rd FAIL" THEN CR ;

: TEST-NIP
    ." Testing NIP..." CR
    5 10 NIP
    10 = IF ." NIP ok" ELSE ." NIP FAIL" THEN CR ;

: TEST-TUCK
    ." Testing TUCK..." CR
    5 10 TUCK
    10 = IF ." Top ok" ELSE ." Top FAIL" THEN CR
    5 = IF ." 2nd ok" ELSE ." 2nd FAIL" THEN CR
    10 = IF ." 3rd ok" ELSE ." 3rd FAIL" THEN CR ;

: TEST-ROT
      ." Testing ROT..." CR
      1 2 3 ROT
      1 = IF ." Top ok (1)" ELSE ." Top FAIL" THEN CR
      3 = IF ." 2nd ok (3)" ELSE ." 2nd FAIL" THEN CR
      2 = IF ." 3rd ok (2)" ELSE ." 3rd FAIL" THEN CR ;

: TEST-2DUP
    1 2 2DUP
    .S CR
    2 = IF ." Top correct (2)" ELSE ." Top wrong!" THEN CR
    1 = IF ." 3rd correct (1)" ELSE ." 3rd wrong!" THEN CR
    2DROP ;

: TEST-2DROP
    1 2 3 4 2DROP
    .S CR
    2 = IF ." Top correct (2)" ELSE ." Top wrong!" THEN CR
    1 = IF ." 2nd correct (1)" ELSE ." 2nd wrong!" THEN CR ;

: TEST-2SWAP
    1 2 3 4 2SWAP
    .S CR
    2 = IF ." Top correct (2)" ELSE ." Top wrong!" THEN CR
    1 = IF ." 2nd correct (1)" ELSE ." 2nd wrong!" THEN CR
    4 = IF ." 3rd correct (4)" ELSE ." 3rd wrong!" THEN CR
    3 = IF ." 4th correct (3)" ELSE ." 4th wrong!" THEN CR ;

: TEST-2OVER
    1 2 3 4 2OVER
    .S CR
    2 = IF ." Top correct (2)" ELSE ." Top wrong!" THEN CR
    1 = IF ." 2nd correct (1)" ELSE ." 2nd wrong!" THEN CR
    4 = IF ." 3rd correct (4)" ELSE ." 3rd wrong!" THEN CR
    3 = IF ." 4th correct (3)" ELSE ." 4th wrong!" THEN CR
    2 = IF ." 5th correct (2)" ELSE ." 5th wrong!" THEN CR
    1 = IF ." 6th correct (1)" ELSE ." 6th wrong!" THEN CR ;

\ =============================================================================
\ ARITHMETIC TESTS
\ =============================================================================

: TEST-NEGATE
    ." Testing NEGATE..." CR
    42 NEGATE -42 = IF ." NEGATE positive ok" ELSE ." FAIL" THEN CR
    -10 NEGATE 10 = IF ." NEGATE negative ok" ELSE ." FAIL" THEN CR ;

: TEST-ABS
    ." Testing ABS..." CR
    -42 ABS 42 = IF ." ABS negative ok" ELSE ." FAIL" THEN CR
    15 ABS 15 = IF ." ABS positive ok" ELSE ." FAIL" THEN CR ;

: TEST-CELLS
    ." Testing CELLS..." CR
    3 CELLS 12 = IF ." CELLS ok" ELSE ." CELLS FAIL" THEN CR ;

: TEST-1+
    ." Testing 1+..." CR
    5 1+ 6 = IF ." 5 1+ ok" ELSE ." 5 1+ FAIL" THEN CR
    0 1+ 1 = IF ." 0 1+ ok" ELSE ." 0 1+ FAIL" THEN CR
    -1 1+ 0 = IF ." -1 1+ ok" ELSE ." -1 1+ FAIL" THEN CR ;

: TEST-1-
    ." Testing 1-..." CR
    5 1- 4 = IF ." 5 1- ok" ELSE ." 5 1- FAIL" THEN CR
    0 1- -1 = IF ." 0 1- ok" ELSE ." 0 1- FAIL" THEN CR
    1 1- 0 = IF ." 1 1- ok" ELSE ." 1 1- FAIL" THEN CR ;

: TEST-2*
    ." Testing 2*..." CR
    5 2* 10 = IF ." 5 2* ok" ELSE ." 5 2* FAIL" THEN CR
    0 2* 0 = IF ." 0 2* ok" ELSE ." 0 2* FAIL" THEN CR
    -3 2* -6 = IF ." -3 2* ok" ELSE ." -3 2* FAIL" THEN CR ;

: TEST-2/
    ." Testing 2/..." CR
    10 2/ 5 = IF ." 10 2/ ok" ELSE ." 10 2/ FAIL" THEN CR
    0 2/ 0 = IF ." 0 2/ ok" ELSE ." 0 2/ FAIL" THEN CR
    -6 2/ -3 = IF ." -6 2/ ok" ELSE ." -6 2/ FAIL" THEN CR ;

: TEST-MIN
    ." Testing MIN..." CR
    5 3 MIN 3 = IF ." 5 3 MIN ok" ELSE ." 5 3 MIN FAIL" THEN CR
    3 5 MIN 3 = IF ." 3 5 MIN ok" ELSE ." 3 5 MIN FAIL" THEN CR
    -2 5 MIN -2 = IF ." -2 5 MIN ok" ELSE ." -2 5 MIN FAIL" THEN CR
    -5 -2 MIN -5 = IF ." -5 -2 MIN ok" ELSE ." -5 -2 MIN FAIL" THEN CR ;

: TEST-MAX
    ." Testing MAX..." CR
    5 3 MAX 5 = IF ." 5 3 MAX ok" ELSE ." 5 3 MAX FAIL" THEN CR
    3 5 MAX 5 = IF ." 3 5 MAX ok" ELSE ." 3 5 MAX FAIL" THEN CR
    -2 5 MAX 5 = IF ." -2 5 MAX ok" ELSE ." -2 5 MAX FAIL" THEN CR
    -5 -2 MAX -2 = IF ." -5 -2 MAX ok" ELSE ." -5 -2 MAX FAIL" THEN CR ;

: TEST-MOD
    ." Testing MOD..." CR
    13 5 MOD 3 = IF ." 13 5 MOD ok" ELSE ." 13 5 MOD FAIL" THEN CR
    10 3 MOD 1 = IF ." 10 3 MOD ok" ELSE ." 10 3 MOD FAIL" THEN CR
    20 7 MOD 6 = IF ." 20 7 MOD ok" ELSE ." 20 7 MOD FAIL" THEN CR
    15 5 MOD 0 = IF ." 15 5 MOD ok (no remainder)" ELSE ." 15 5 MOD FAIL" THEN CR ;

\ =============================================================================
\ IO TESTS
\ =============================================================================

: TEST-SPACE
    ." Testing SPACE (should see one space between brackets):["
    SPACE
    ." ]" CR ;

: TEST-SPACES
    ." Testing SPACES (should see 5 spaces):["
    5 SPACES
    ." ]" CR ;

\ =============================================================================
\ MEMORY TESTS
\ =============================================================================

: TEST-CELL+
    ." Testing CELL+..." CR
    0 CELL+ 4 = IF ." 0 CELL+ ok" ELSE ." 0 CELL+ FAIL" THEN CR
    100 CELL+ 104 = IF ." 100 CELL+ ok" ELSE ." 100 CELL+ FAIL" THEN CR
    131072 CELL+ 131076 = IF ." 131072 CELL+ ok" ELSE ." 131072 CELL+ FAIL" THEN CR ;

: TEST-+!
    ." Testing +!..." CR
    131072 \ addr

    \ Store 42
    42 OVER !

    \ Add 10 to it
    10 OVER +!

    \ Fetch and verify (should be 52)
    DUP @ 52 = IF ." +! positive ok" ELSE ." +! positive FAIL" THEN CR

    \ Add -5 to it
    -5 OVER +!

    \ Fetch and verify (should be 47)
    DUP @ 47 = IF ." +! negative ok" ELSE ." +! negative FAIL" THEN CR

    \ Test adding 0
    0 OVER +!
    @ 47 = IF ." +! zero ok" ELSE ." +! zero FAIL" THEN CR ;

\ =============================================================================
\ TEST RUNNERS
\ =============================================================================

: RUN-ALL-TESTS
    ." " CR
    ." ============================================" CR
    ." RUNNING ALL STDLIB TESTS" CR
    ." ============================================" CR
    ." " CR

    TEST-0=
    TEST-=
    TEST-0<
    TEST-0>
    TEST-ROT
    TEST-<>
    TEST-<=
    TEST->=
    TEST-CONSTANTS
    TEST-OVER
    TEST-NIP
    TEST-TUCK
    ." Running 2DUP test..." CR TEST-2DUP ." PASSED" CR CR
    ." Running 2DROP test..." CR TEST-2DROP ." PASSED" CR CR
    ." Running 2SWAP test..." CR TEST-2SWAP ." PASSED" CR CR
    ." Running 2OVER test..." CR TEST-2OVER ." PASSED" CR CR
    TEST-NEGATE
    TEST-ABS
    TEST-CELLS
    TEST-1+
    TEST-1-
    TEST-2*
    TEST-2/
    TEST-MIN
    TEST-MAX
    TEST-MOD
    TEST-SPACE
    TEST-SPACES
    TEST-CELL+
    TEST-+!

    ." " CR
    ." ============================================" CR
    ." ALL STDLIB TESTS PASSED!" CR
    ." ============================================" CR ;
