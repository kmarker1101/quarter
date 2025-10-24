\ Performance benchmark: Factorial
\ Calculate 12! (479,001,600 - fits in i32)
\ Expected: 100x speedup with optimizations

: FACTORIAL ( n -- n! )
    DUP 1 <= IF
        DROP 1
    ELSE
        DUP 1 - FACTORIAL *
    THEN ;

\ Run the benchmark
12 FACTORIAL .
