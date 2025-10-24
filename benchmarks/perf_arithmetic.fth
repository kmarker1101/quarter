\ Performance benchmark: Arithmetic loop
\ Start with 100,000 iterations (scale up after optimization)
\ Expected: 1000x speedup with optimizations

: BENCH-ARITHMETIC ( -- result )
    0                    \ accumulator
    100000 0 DO
        I 2 * 3 + 4 - 5 / DROP
        1 +              \ increment accumulator
    LOOP ;

\ Run the benchmark
BENCH-ARITHMETIC .
