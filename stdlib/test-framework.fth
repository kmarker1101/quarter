\ Quarter Forth Test Framework - Simplified
\ T{ -> }T syntax for testing
\
\ Usage:
\   TESTING
\   T{ 5 3 + -> 8 }T
\   S" Addition with DUP" TEST:
\   T{ 5 DUP -> 5 5 }T
\   REPORT

VARIABLE #PASS
VARIABLE #FAIL
VARIABLE #TEST
VARIABLE ACTUAL-DEPTH
VARIABLE TEST-NAME-ADDR
VARIABLE TEST-NAME-LEN
VARIABLE TEST-PTR

HERE CONSTANT TEST-STORAGE
256 ALLOT
HERE CONSTANT EXPECTED-STORAGE
256 ALLOT

\ Initialize testing
: TESTING
    CR
    ." Running tests..." CR
    0 #PASS !
    0 #FAIL !
    0 #TEST !
    0 TEST-NAME-LEN ! ;

\ Set name for next test
: TEST: ( addr len -- )
    TEST-NAME-LEN !
    TEST-NAME-ADDR ! ;

\ Start test - reset storage pointer
: T{
    TEST-STORAGE TEST-PTR !
    1 #TEST +! ;

\ Separator - save actual values to memory
: -> ( actuals... --  )
    DEPTH DUP ACTUAL-DEPTH !
    \ Save count on return stack, then store values in reverse order
    DUP >R
    0 DO
        TEST-PTR @ !
        TEST-PTR @ CELL+ TEST-PTR !
    LOOP
    R> DROP ;

\ Print test identifier (name or number)
: .TEST-ID
    TEST-NAME-LEN @ 0 > IF
        TEST-NAME-ADDR @ TEST-NAME-LEN @ TYPE
    ELSE
        ." Test " #TEST @ .
    THEN ;

\ End test - compare expected with actual
: }T ( expecteds... -- )
    DEPTH ACTUAL-DEPTH @ = IF
        \ Save expected values to EXPECTED-STORAGE (pops from stack)
        ACTUAL-DEPTH @ 0 DO
            EXPECTED-STORAGE I CELLS + !
        LOOP

        \ Now compare with stored actuals
        TRUE
        ACTUAL-DEPTH @ 0 DO
            \ Get stored actual
            TEST-STORAGE I CELLS + @
            \ Get expected from storage
            EXPECTED-STORAGE I CELLS + @
            \ Compare
            = AND
        LOOP
        IF
            \ Test passed - just increment counter
            1 #PASS +!
        ELSE
            \ Test failed - show expected vs actual
            1 #FAIL +!
            CR
            ." FAIL: " .TEST-ID CR
            ."   Expected: "
            ACTUAL-DEPTH @ 0 DO
                EXPECTED-STORAGE I CELLS + @ .
            LOOP CR
            ."   Actual:   "
            ACTUAL-DEPTH @ 0 DO
                TEST-STORAGE I CELLS + @ .
            LOOP CR
        THEN
    ELSE
        \ Wrong number of values
        1 #FAIL +!
        CR
        ." FAIL: " .TEST-ID ."  (wrong number of values)" CR
        ."   Expected " ACTUAL-DEPTH @ . ." values, got " DEPTH . CR
        \ Clean stack
        DEPTH 0 DO DROP LOOP
    THEN
    \ Clear test name for next test
    0 TEST-NAME-LEN ! ;

\ Print summary
: REPORT
    CR
    ." ================================" CR
    ." Total:  " #PASS @ #FAIL @ + . CR
    ." Passed: " #PASS @ . CR
    ." Failed: " #FAIL @ . CR
    ." ================================" CR ;
