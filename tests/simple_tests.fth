\ Simple single-value tests
\ Load with: ./quarter tests/simple_tests.fth

TESTING

\ Basic arithmetic
T{ 5 3 + -> 8 }T
T{ 10 2 - -> 8 }T
T{ 4 5 * -> 20 }T
T{ 20 4 / -> 5 }T

\ Stack operations
T{ 5 DROP 10 -> 10 }T

\ Comparisons (Forth TRUE = -1, FALSE = 0)
T{ TRUE -> -1 }T
T{ FALSE -> 0 }T

REPORT
