\ Performance benchmark: Fibonacci
\ Calculate Fibonacci(30)
\ Expected: 1000x speedup with optimizations

: FIBONACCI ( n -- fib[n] )
    DUP 2 < IF
        DROP 1
    ELSE
        DUP 1 - RECURSE
        SWAP 2 - RECURSE
        +
    THEN ;

\ Run the benchmark
30 FIBONACCI .
