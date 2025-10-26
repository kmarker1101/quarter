\ Performance benchmark: Factorial for Gforth
\ Calculate 12! (479,001,600)

: FACTORIAL ( n -- n! )
    DUP 1 <= IF
        DROP 1
    ELSE
        DUP 1 - FACTORIAL *
    THEN ;

\ Run the benchmark
12 FACTORIAL . CR
