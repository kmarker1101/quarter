\ Simple Arithmetic Benchmarks
\ These benchmarks test JIT-compilable arithmetic operations

\ Benchmark 1: Simple addition chain (JIT-compilable)
: ADDCHAIN 1 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + ;

\ Benchmark 2: SQUARE using DUP and * (JIT-compilable)
: SQUARE DUP * ;

\ Benchmark 3: DOUBLE using DUP and + (JIT-compilable)
: DOUBLE DUP + ;

\ Benchmark 4: Complex arithmetic (JIT-compilable)
: COMPLEX 5 DUP * 3 DUP * + ;  \ 5^2 + 3^2 = 34

\ Benchmark 5: Division chain (JIT-compilable)
: DIVCHAIN 1000 2 / 2 / 2 / ;  \ 1000 / 8 = 125

\ Test the words
ADDCHAIN
7 SQUARE
10 DOUBLE
COMPLEX
DIVCHAIN
