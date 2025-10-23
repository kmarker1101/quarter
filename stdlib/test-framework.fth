\ Quarter Forth Test Framework - Simplified
\ T{ -> }T syntax for testing
\
\ Usage:
\   TESTING
\   T{ 5 3 + -> 8 }T
\   T{ 5 DUP -> 5 5 }T
\   REPORT

VARIABLE #PASS
VARIABLE #FAIL
VARIABLE ACTUAL-DEPTH

131072 CONSTANT TEST-STORAGE
VARIABLE TEST-PTR

\ Initialize testing
: TESTING 0 #PASS ! 0 #FAIL ! ;

\ Start test - reset storage pointer
: T{ TEST-STORAGE TEST-PTR ! ;

\ Separator - save actual values to memory
: -> ( actuals... --  )
    DEPTH DUP ACTUAL-DEPTH !
    \ Store each value
    0 DO
        TEST-PTR @ !
        TEST-PTR @ 4 + TEST-PTR !
    LOOP ;

\ End test - compare expected with actual
: }T ( expecteds... -- )
    DEPTH ACTUAL-DEPTH @ = IF
        \ Right number of values
        TRUE
        ACTUAL-DEPTH @ 0 DO
            \ Get stored actual (in reverse)
            TEST-STORAGE ACTUAL-DEPTH @ I - 1 - 4 * + @
            \ Compare with expected
            = AND
        LOOP
        IF
            1 #PASS +!
        ELSE
            1 #FAIL +!
        THEN
    ELSE
        \ Wrong number
        1 #FAIL +!
        \ Clean stack
        DEPTH 0 DO DROP LOOP
    THEN ;

\ Print summary
: REPORT
    CR
    #PASS @ #FAIL @ + .
    #PASS @ .
    #FAIL @ .
    CR ;
