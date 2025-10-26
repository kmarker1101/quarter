\ Portable arithmetic benchmark
\ Works with both Quarter and Gforth
\ Pure arithmetic loop - no recursion needed

: BENCH-ARITHMETIC ( -- result )
    0                           \ accumulator
    100000 0 DO
        I 2 * 3 + 4 - 5 / DROP
        1 +                     \ increment accumulator
    LOOP ;

\ Run and print result
BENCH-ARITHMETIC .
