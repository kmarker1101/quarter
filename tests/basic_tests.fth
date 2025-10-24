\ Basic tests for Quarter Forth primitives
\ Load with: ./quarter tests/basic_tests.fth

S" stdlib/test-framework.fth" INCLUDED

\ =============================================================================
\ ARITHMETIC TESTS
\ =============================================================================

TESTING

S" Addition: 5 + 3" TEST:
T{ 5 3 + -> 8 }T

S" Subtraction: 10 - 2" TEST:
T{ 10 2 - -> 8 }T

S" Multiplication: 4 * 5" TEST:
T{ 4 5 * -> 20 }T

S" Division: 20 / 4" TEST:
T{ 20 4 / -> 5 }T

S" 13 MOD 5 equals 3" TEST:
T{ 13 5 MOD -> 3 }T

S" 10 MOD 3 equals 1" TEST:
T{ 10 3 MOD -> 1 }T

S" 20 MOD 7 equals 6" TEST:
T{ 20 7 MOD -> 6 }T

S" 15 MOD 5 equals 0" TEST:
T{ 15 5 MOD -> 0 }T
  
S" Divide and modulo: 13 /MOD 5" TEST:
T{ 13 5 /MOD -> 3 2 }T

S" 3 CELLS equals 12" TEST:
T{ 3 CELLS -> 12 }T

S" 5 1+ equals 6" TEST:
T{ 5 1+ -> 6 }T

S" 0 1+ equals 1" TEST:
T{ 0 1+ -> 1 }T

S" -1 1+ equals 0" TEST:
T{ -1 1+ -> 0 }T

S" 5 1- equals 4" TEST:
T{ 5 1- -> 4 }T

S" 0 1- equals -1" TEST:
T{ 0 1- -> -1 }T

S" 1 1- equals 0" TEST:
T{ 1 1- -> 0 }T

S" 5 2* equals 10" TEST:
T{ 5 2* -> 10 }T

S" 0 2* equals 0" TEST:
T{ 0 2* -> 0 }T

S" -3 2* equals -6" TEST:
T{ -3 2* -> -6 }T

S" 10 2/ equals 5" TEST:
T{ 10 2/ -> 5 }T

S" 0 2/ equals 0" TEST:
T{ 0 2/ -> 0 }T

S" -6 2/ equals -3" TEST:
T{ -6 2/ -> -3 }T

\ =============================================================================
\ STACK MANIPULATION TESTS
\ =============================================================================

S" DUP duplicates top value" TEST:
T{ 5 DUP -> 5 5 }T

S" SWAP exchanges top two values" TEST:
T{ 5 10 SWAP -> 10 5 }T

S" DROP removes top value" TEST:
T{ 5 10 DROP -> 5 }T

S" OVER copies second to top" TEST:
T{ 5 10 OVER -> 5 10 5 }T

S" NIP removes second value" TEST:
T{ 5 10 NIP -> 10 }T

S" TUCK inserts copy under second" TEST:
T{ 5 10 TUCK -> 10 5 10 }T

S" ROT rotates top 3 entries" TEST:
T{ 1 2 3 ROT -> 2 3 1 }T

S" 2DUP duplicates top 2 values" TEST:
T{ 1 2 2DUP -> 1 2 1 2 }T

S" 2DROP removes top 2 values" TEST:
T{ 1 2 3 4 2DROP -> 1 2 }T

S" 2SWAP exchanges top two pairs" TEST:
T{ 1 2 3 4 2SWAP -> 3 4 1 2 }T

S" 2OVER copies 2nd pair to top" TEST:
T{ 1 2 3 4 2OVER -> 1 2 3 4 1 2 }T
\ =============================================================================
\ COMPARISON TESTS
\ =============================================================================

S" 3 < 5 is false" TEST:
T{ 5 3 < -> 0 }T

S" 5 < 3 is true" TEST:
T{ 3 5 < -> -1 }T

S" 5 < 5 is false" TEST:
T{ 5 5 < -> 0 }T

S" 3 > 5 is true" TEST:
T{ 5 3 > -> -1 }T

S" 5 > 3 is false" TEST:
T{ 3 5 > -> 0 }T

S" 5 > 5 is false" TEST:
T{ 5 5 > -> 0 }T

S" 5 = 5 is true" TEST:
T{ 5 5 = -> -1 }T

S" 5 = 3 is false" TEST:
T{ 5 3 = -> 0 }T

S" 0 equals zero" TEST:
T{ 0 0= -> -1 }T

S" 5 does not equal zero" TEST:
T{ 5 0= -> 0 }T

S" -3 does not equal zero" TEST:
T{ -3 0= -> 0 }T

S" 5 equals 5" TEST:
T{ 5 5 = -> -1 }T

S" 5 does not equal 3" TEST:
T{ 5 3 = -> 0 }T

S" -2 equals -2" TEST:
T{ -2 -2 = -> -1 }T

S" -5 is less than zero" TEST:
T{ -5 0< -> -1 }T

S" 0 is not less than zero" TEST:
T{ 0 0< -> 0 }T

S" 5 is not less than zero" TEST:
T{ 5 0< -> 0 }T

S" 5 is greater than zero" TEST:
T{ 5 0> -> -1 }T

S" 0 is not greater than zero" TEST:
T{ 0 0> -> 0 }T

S" -5 is not greater than zero" TEST:
T{ -5 0> -> 0 }T

S" 5 is not equal to 3" TEST:
T{ 5 3 <> -> -1 }T

S" 5 equals 5" TEST:
T{ 5 5 <> -> 0 }T

S" -2 is not equal to 3" TEST:
T{ -2 3 <> -> -1 }T

S" 3 <= 5 is true" TEST:
T{ 3 5 <= -> -1 }T

S" 5 <= 5 is true" TEST:
T{ 5 5 <= -> -1 }T

S" 5 <= 3 is false" TEST:
T{ 5 3 <= -> 0 }T

S" -5 <= 0 is true" TEST:
T{ -5 0 <= -> -1 }T

S" 5 >= 3 is true" TEST:
T{ 5 3 >= -> -1 }T

S" 5 >= 5 is true" TEST:
T{ 5 5 >= -> -1 }T

S" 3 >= 5 is false" TEST:
T{ 3 5 >= -> 0 }T

S" 0 >= -5 is true" TEST:
T{ 0 -5 >= -> -1 }T

\ =============================================================================
\ BITWISE TESTS
\ =============================================================================

S" Bitwise AND: 5 AND 3" TEST:
T{ 5 3 AND -> 1 }T

S" Bitwise OR: 5 OR 3" TEST:
T{ 5 3 OR -> 7 }T

S" Bitwise XOR: 5 XOR 3" TEST:
T{ 5 3 XOR -> 6 }T

S" Bitwise INVERT of -1" TEST:
T{ -1 INVERT -> 0 }T

S" Left shift: 1 LSHIFT 2" TEST:
T{ 1 2 LSHIFT -> 4 }T

S" Right shift: 8 RSHIFT 2" TEST:
T{ 8 2 RSHIFT -> 2 }T

\ =============================================================================
\ MEMORY TESTS
\ =============================================================================

VARIABLE TEST-VAR

S" Store and fetch 42 from variable" TEST:
T{ 42 TEST-VAR ! TEST-VAR @ -> 42 }T

S" Store and fetch 100 from variable" TEST:
T{ 100 TEST-VAR ! TEST-VAR @ -> 100 }T

\ =============================================================================
\ CONTROL STRUCTURE TESTS
\ =============================================================================

: TEST-IF-POSITIVE DUP 0 SWAP > IF 10 + ELSE 20 + THEN ;

S" IF-THEN-ELSE with positive number" TEST:
T{ 5 TEST-IF-POSITIVE -> 25 }T

S" IF-THEN-ELSE with negative number" TEST:
T{ -5 TEST-IF-POSITIVE -> 5 }T

\ =============================================================================
\ STDLIB TESTS
\ =============================================================================

S" NEGATE positive number" TEST:
T{ 5 NEGATE -> -5 }T

S" NEGATE negative number" TEST:
T{ -10 NEGATE -> 10 }T

S" ABS of negative number" TEST:
T{ -5 ABS -> 5 }T

S" ABS of positive number" TEST:
T{ 5 ABS -> 5 }T

S" Increment by 1" TEST:
T{ 5 1+ -> 6 }T

S" Decrement by 1" TEST:
T{ 5 1- -> 4 }T

S" Multiply by 2" TEST:
T{ 5 2* -> 10 }T

S" Divide by 2" TEST:
T{ 10 2/ -> 5 }T

S" MIN returns smaller value" TEST:
T{ 5 3 MIN -> 3 }T

S" MAX returns larger value" TEST:
T{ 5 3 MAX -> 5 }T

S" ROT rotates three values" TEST:
T{ 1 2 3 ROT -> 2 3 1 }T

S" TRUE equals -1" TEST:
T{ TRUE -> -1 }T

S" FALSE equals 0" TEST:
T{ FALSE -> 0 }T

S" BL equals 32" TEST:
T{ BL -> 32 }T

S" 0 CELL+ equals 4" TEST:
T{ 0 CELL+ -> 4 }T

S" 100 CELL+ equals 104" TEST:
T{ 100 CELL+ -> 104 }T

S" 131072 CELL+ equals 131076" TEST:
T{ 131072 CELL+ -> 131076 }T

VARIABLE +!-TEST

S" +! adds 10 to stored 42" TEST:
T{ 42 +!-TEST ! 10 +!-TEST +! +!-TEST @ -> 52 }T

S" +! adds -5 to stored value" TEST:
T{ 52 +!-TEST ! -5 +!-TEST +! +!-TEST @ -> 47 }T

S" +! adds 0 to stored value" TEST:
T{ 47 +!-TEST ! 0 +!-TEST +! +!-TEST @ -> 47 }T
\ =============================================================================
\ REPORT
\ =============================================================================

REPORT
