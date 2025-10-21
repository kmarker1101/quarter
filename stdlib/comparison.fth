\ Quarter Forth Standard Library - Comparison Operations
\ Requires: core.fth (for FALSE, TRUE)

: 0= ( n -- flag ) IF FALSE ELSE TRUE THEN ;

: 0< ( n -- flag ) 0 < ;

: 0> ( n -- flag ) 0 > ;

: = ( n1 n2 -- flag ) - 0= ;

: <> ( n1 n2 -- flag ) = 0= ;

: <= ( n1 n2 -- flag ) > 0= ;

: >= ( n1 n2 -- flag ) < 0= ;

\ -----------------------------------------------------------------------------
\ Test Suite
\ -----------------------------------------------------------------------------

: TEST-0=
    ." Testing 0=..." CR
    0 0= TRUE = IF ." 0 0= ok" ELSE ." 0 0= FAIL" THEN CR
    5 0= FALSE = IF ." 5 0= ok" ELSE ." 5 0= FAIL" THEN CR
    -3 0= FALSE = IF ." -3 0= ok" ELSE ." -3 0= FAIL" THEN CR ;

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

: TEST-=
    ." Testing =..." CR
    5 5 = TRUE = IF ." 5 5 = ok" ELSE ." 5 5 = FAIL" THEN CR
    5 3 = FALSE = IF ." 5 3 = ok" ELSE ." 5 3 = FAIL" THEN CR
    -2 -2 = TRUE = IF ." -2 -2 = ok" ELSE ." -2 -2 = FAIL" THEN CR ;

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

: RUN-COMPARISON-TESTS
    TEST-0=
    TEST-0<
    TEST-0>
    TEST-=
    TEST-<>
    TEST-<=
    TEST->=
    ." All comparison tests passed!" CR ;

\ Uncomment to auto-run tests when loading this file:
\ RUN-COMPARISON-TESTS
