\ Performance benchmark: Fibonacci
\ Calculate Fibonacci(30)
\ Expected: 1000x speedup with optimizations

: FIBONACCI ( n -- fib[n] )
    DUP 2 < IF
        DROP 1
    ELSE
        DUP 1 - FIBONACCI
        SWAP 2 - FIBONACCI
        +
    THEN ;

\ Run the benchmark
30 FIBONACCI .
