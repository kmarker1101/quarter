\ Performance benchmark: Nested loops
\ 1000x1000 nested loops
\ Expected: 1000x speedup with optimizations

: BENCH-LOOP-NEST ( -- )
    1000 0 DO
        1000 0 DO
            I J + DROP
        LOOP
    LOOP ;

\ Run the benchmark
BENCH-LOOP-NEST
