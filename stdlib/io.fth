: SPACE BL EMIT ;

\ Need to implement ?DO
\ then this becomes
\ : SPACES ( n -- ) 0 ?DO SPACE LOOP ;)
: SPACES ( n -- )
    DUP 0 > IF
      0 DO SPACE LOOP
    ELSE
      DROP
    THEN ;

\ -----------------------------------------------------------------------------
\ Test Suite
\ -----------------------------------------------------------------------------

: TEST-SPACE
    ." Testing SPACE (should see one space between brackets):["
    SPACE
    ." ]" CR ;

: TEST-SPACES
    ." Testing SPACES (should see 5 spaces):["
    5 SPACES
    ." ]" CR ;

: RUN-IO-TESTS
    TEST-SPACE
    TEST-SPACES
    ." All IO tests passed!" CR ;

\ Uncomment to auto-run tests when loading this file:
\ RUN-IO-TESTS
