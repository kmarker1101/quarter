0 CONSTANT FALSE

-1 CONSTANT TRUE

32 CONSTANT BL

: OVER  ( x1 x2 -- x1 x2 x1 ) >R DUP R> SWAP ;

: NIP ( n1 n2 -- n2 ) SWAP DROP ;

: TUCK ( n1 n2 -- n2 n1 n2 ) SWAP OVER ;

: NEGATE ( n1 -- n2 ) 0 SWAP - ;

: ABS ( n -- +n ) DUP 0 < IF NEGATE THEN ;

: CELLS ( n -- n ) 4 * ;

\ -----------------------------------------------------------------------------
\ Test Suite
\ -----------------------------------------------------------------------------

: TEST-CONSTANTS
    ." Testing constants..." CR
    TRUE -1 = IF ." TRUE ok" ELSE ." TRUE FAIL" THEN CR
    FALSE 0 = IF ." FALSE ok" ELSE ." FALSE FAIL" THEN CR
    BL 32 = IF ." BL ok" ELSE ." BL FAIL" THEN CR ;

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

: RUN-CORE-TESTS
    TEST-CONSTANTS
    TEST-OVER
    TEST-NIP
    TEST-TUCK
    TEST-NEGATE
    TEST-ABS
    TEST-CELLS
    ." All core tests passed!" CR ;

\ Uncomment to auto-run tests when loading this file:
\ RUN-CORE-TESTS
