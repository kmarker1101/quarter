\ Quarter Forth Standard Library - Math Operations
\ Requires: core.fth, stack.fth (for 2DUP)

: 1+ ( n -- n+1 ) 1 + ;

: 1- ( n -- n-1 ) 1 - ;

: 2* ( n -- n*2 ) 2 * ;

: 2/ ( n -- n/2 ) 2 / ;

: MIN ( n1 n2 -- n ) 2DUP > IF SWAP THEN DROP ;

: MAX ( n1 n2 -- n ) 2DUP < IF SWAP THEN DROP ;

\ -----------------------------------------------------------------------------
\ Test Suite
\ -----------------------------------------------------------------------------

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

: RUN-MATH-TESTS
    TEST-1+
    TEST-1-
    TEST-2*
    TEST-2/
    TEST-MIN
    TEST-MAX
    ." All math tests passed!" CR ;

\ Uncomment to auto-run tests when loading this file:
\ RUN-MATH-TESTS
