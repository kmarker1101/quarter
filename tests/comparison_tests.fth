\ Comparison operator tests for Quarter Forth
\
\ Interpreted mode: ./quarter tests/comparison_tests.fth
\ JIT mode (via run-all-tests.fth): ./quarter tests/run-all-tests.fth --jit

\ Load test framework if not already loaded
S" stdlib/test-framework.fth" INCLUDED

TESTING

\ =============================================================================
\ BASIC COMPARISON TESTS
\ =============================================================================

S" 3 < 5 (true)" TEST:
T{ 3 5 < -> -1 }T

S" 5 < 3 (false)" TEST:
T{ 5 3 < -> 0 }T

S" 3 < 3 (equal, false)" TEST:
T{ 3 3 < -> 0 }T

S" 5 > 3 (true)" TEST:
T{ 5 3 > -> -1 }T

S" 3 > 5 (false)" TEST:
T{ 3 5 > -> 0 }T

S" 3 > 3 (equal, false)" TEST:
T{ 3 3 > -> 0 }T

S" 5 = 5 (true)" TEST:
T{ 5 5 = -> -1 }T

S" 5 = 3 (false)" TEST:
T{ 5 3 = -> 0 }T

S" 5 <> 3 (true)" TEST:
T{ 5 3 <> -> -1 }T

S" 5 <> 5 (false)" TEST:
T{ 5 5 <> -> 0 }T

\ =============================================================================
\ LESS OR EQUAL / GREATER OR EQUAL
\ =============================================================================

S" 3 <= 5 (less, true)" TEST:
T{ 3 5 <= -> -1 }T

S" 5 <= 5 (equal, true)" TEST:
T{ 5 5 <= -> -1 }T

S" 5 <= 3 (greater, false)" TEST:
T{ 5 3 <= -> 0 }T

S" 5 >= 3 (greater, true)" TEST:
T{ 5 3 >= -> -1 }T

S" 5 >= 5 (equal, true)" TEST:
T{ 5 5 >= -> -1 }T

S" 3 >= 5 (less, false)" TEST:
T{ 3 5 >= -> 0 }T

\ =============================================================================
\ NEGATIVE NUMBER COMPARISONS
\ =============================================================================

S" -5 < -3 (negative numbers)" TEST:
T{ -5 -3 < -> -1 }T

S" -3 > -5 (negative numbers)" TEST:
T{ -3 -5 > -> -1 }T

S" -3 < 5 (negative vs positive)" TEST:
T{ -3 5 < -> -1 }T

S" 5 > -3 (positive vs negative)" TEST:
T{ 5 -3 > -> -1 }T

S" -3 = -3 (negative equality)" TEST:
T{ -3 -3 = -> -1 }T

S" -3 <> 3 (different sign)" TEST:
T{ -3 3 <> -> -1 }T

\ =============================================================================
\ ZERO COMPARISON TESTS
\ =============================================================================

S" 0 0= (zero equals zero)" TEST:
T{ 0 0= -> -1 }T

S" 5 0= (positive not zero)" TEST:
T{ 5 0= -> 0 }T

S" -3 0= (negative not zero)" TEST:
T{ -3 0= -> 0 }T

S" -5 0< (negative less than zero)" TEST:
T{ -5 0< -> -1 }T

S" 0 0< (zero not less than zero)" TEST:
T{ 0 0< -> 0 }T

S" 5 0< (positive not less than zero)" TEST:
T{ 5 0< -> 0 }T

S" 5 0> (positive greater than zero)" TEST:
T{ 5 0> -> -1 }T

S" 0 0> (zero not greater than zero)" TEST:
T{ 0 0> -> 0 }T

S" -5 0> (negative not greater than zero)" TEST:
T{ -5 0> -> 0 }T

\ =============================================================================
\ STACK OPERATION TESTS
\ =============================================================================

S" PICK with index 0 (top)" TEST:
T{ 1 2 3 0 PICK -> 1 2 3 3 }T

S" PICK with index 1 (second)" TEST:
T{ 1 2 3 1 PICK -> 1 2 3 2 }T

S" PICK with index 2 (third)" TEST:
T{ 1 2 3 2 PICK -> 1 2 3 1 }T

S" DEPTH on empty stack" TEST:
T{ DEPTH -> 0 }T

S" DEPTH with one item" TEST:
T{ 42 DEPTH -> 42 1 }T

S" DEPTH with three items" TEST:
T{ 1 2 3 DEPTH -> 1 2 3 3 }T

\ =============================================================================
\ REPORT
\ =============================================================================

REPORT
