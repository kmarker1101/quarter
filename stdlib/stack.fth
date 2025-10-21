\ Stack manipulation words for double-cell operations
\ Part of Quarter Forth Standard Library

\ 2DUP ( a b -- a b a b )
\ Duplicate the top two stack items
: 2DUP OVER OVER ;

\ 2DROP ( a b -- )
\ Drop the top two stack items
: 2DROP DROP DROP ;

\ 2SWAP ( a b c d -- c d a b )
\ Swap the top two pairs of stack items
: 2SWAP ROT >R ROT R> ;

\ 2OVER ( a b c d -- a b c d a b )
\ Copy the second pair of stack items to the top
: 2OVER >R >R 2DUP R> R> 2SWAP ;

\ -----------------------------------------------------------------------------
\ Test Suite
\ -----------------------------------------------------------------------------

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

\ Run all tests
: RUN-STACK-TESTS
    ." Running 2DUP test..." CR TEST-2DUP ." PASSED" CR CR
    ." Running 2DROP test..." CR TEST-2DROP ." PASSED" CR CR
    ." Running 2SWAP test..." CR TEST-2SWAP ." PASSED" CR CR
    ." Running 2OVER test..." CR TEST-2OVER ." PASSED" CR CR
    ." All stack tests passed!" CR ;

\ Uncomment to auto-run tests when loading this file:
\ RUN-STACK-TESTS
