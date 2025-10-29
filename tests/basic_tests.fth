\ Basic tests for Quarter Forth primitives
\
\ Interpreted mode: ./quarter tests/basic_tests.fth
\ JIT mode: ./quarter tests/run-all-tests.fth --jit

\ Load test framework if not already loaded
\ (run-all-tests.fth loads it once for all tests)
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

S" /MOD with exact division" TEST:
T{ 15 3 /MOD -> 0 5 }T

S" /MOD with larger numbers" TEST:
T{ 100 7 /MOD -> 2 14 }T

S" /MOD with negative dividend" TEST:
T{ -17 5 /MOD -> -2 -3 }T

S" /MOD edge case: 20 6" TEST:
T{ 20 6 /MOD -> 2 3 }T

S" 3 CELLS equals 24" TEST:
T{ 3 CELLS -> 24 }T

S" HERE returns initial dictionary pointer" TEST:
T{ HERE 131072 >= -> -1 }T

S" ALLOT advances HERE by n bytes" TEST:
: TEST-ALLOT-HERE HERE 16 ALLOT HERE SWAP - ;
T{ TEST-ALLOT-HERE -> 16 }T

S" Comma advances HERE by 8 bytes" TEST:
: TEST-COMMA-HERE HERE 42 , HERE SWAP - ;
T{ TEST-COMMA-HERE -> 8 }T

S" Comma stores value and HERE advances" TEST:
: TEST-COMMA-STORE-FETCH HERE DUP 777 , @ ;
T{ TEST-COMMA-STORE-FETCH -> 777 }T

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

S" MIN with negative numbers" TEST:
T{ -5 -10 MIN -> -10 }T

S" MIN with mixed signs" TEST:
T{ -5 3 MIN -> -5 }T

S" MIN with equal values" TEST:
T{ 7 7 MIN -> 7 }T

S" MIN with zero" TEST:
T{ 0 5 MIN -> 0 }T

S" MAX returns larger value" TEST:
T{ 5 3 MAX -> 5 }T

S" MAX with negative numbers" TEST:
T{ -5 -10 MAX -> -5 }T

S" MAX with mixed signs" TEST:
T{ -5 3 MAX -> 3 }T

S" MAX with equal values" TEST:
T{ 7 7 MAX -> 7 }T

S" MAX with zero" TEST:
T{ 0 5 MAX -> 5 }T

S" ABS of zero" TEST:
T{ 0 ABS -> 0 }T

S" ROT rotates three values" TEST:
T{ 1 2 3 ROT -> 2 3 1 }T

S" TRUE equals -1" TEST:
T{ TRUE -> -1 }T

S" FALSE equals 0" TEST:
T{ FALSE -> 0 }T

S" BL equals 32" TEST:
T{ BL -> 32 }T

S" 0 CELL+ equals 8" TEST:
T{ 0 CELL+ -> 8 }T

S" 100 CELL+ equals 108" TEST:
T{ 100 CELL+ -> 108 }T

S" 131072 CELL+ equals 131080" TEST:
T{ 131072 CELL+ -> 131080 }T

VARIABLE +!-TEST

S" +! adds 10 to stored 42" TEST:
T{ 42 +!-TEST ! 10 +!-TEST +! +!-TEST @ -> 52 }T

S" +! adds -5 to stored value" TEST:
T{ 52 +!-TEST ! -5 +!-TEST +! +!-TEST @ -> 47 }T

S" +! adds 0 to stored value" TEST:
T{ 47 +!-TEST ! 0 +!-TEST +! +!-TEST @ -> 47 }T

\ =============================================================================
\ INLINE PRIMITIVE TESTS (Issue #60 - 12 High Priority Primitives)
\ =============================================================================

\ 1+ tests (additional edge cases)
S" Large number 1+" TEST:
T{ 1000000 1+ -> 1000001 }T

S" Negative number 1+" TEST:
T{ -100 1+ -> -99 }T

\ 1- tests (additional edge cases)
S" Large number 1-" TEST:
T{ 1000000 1- -> 999999 }T

S" Negative number 1-" TEST:
T{ -100 1- -> -101 }T

\ 2* tests (additional)
S" Large number 2*" TEST:
T{ 500 2* -> 1000 }T

\ 2/ tests (additional)
S" Odd number 2/" TEST:
T{ 11 2/ -> 5 }T

S" Negative odd 2/" TEST:
T{ -11 2/ -> -5 }T

\ MIN tests (comprehensive already added earlier)
\ MAX tests (comprehensive already added earlier)
\ ABS tests (comprehensive already added earlier)

\ =============================================================================
\ RETURN STACK TESTS (>R, R>, R@)
\ =============================================================================

S" >R and R> basic" TEST:
T{ 42 >R R> -> 42 }T

S" >R and R> with addition" TEST:
T{ 10 20 >R R> + -> 30 }T

S" R@ peeks without popping" TEST:
T{ 99 >R R@ R> + -> 198 }T

S" >R R> order (LIFO)" TEST:
T{ 1 >R 2 >R 3 >R R> R> R> -> 3 2 1 }T

S" Complex return stack usage" TEST:
T{ 5 >R 10 >R R> R> + -> 15 }T

S" R@ doesn't modify return stack" TEST:
T{ 42 >R R@ DROP R> -> 42 }T

\ =============================================================================
\ LOOP INDEX TESTS (I and J)
\ =============================================================================

\ Note: Loop tests with I and J are in tests/word_tests.rs
\ because they require DO/LOOP which is compile-only and cannot
\ be used in the test framework's interpreted mode.

\ =============================================================================
\ MULTIPLY-DIVIDE (*/) TESTS
\ =============================================================================

TESTING

S" */ basic operation" TEST:
T{ 6 7 2 */ -> 21 }T

S" */ with large intermediate" TEST:
T{ 1000000 1000000 2 */ -> 500000000000 }T

S" */ exact division" TEST:
T{ 12 15 3 */ -> 60 }T

S" */ with negative numbers" TEST:
T{ -10 6 3 */ -> -20 }T

S" */ preventing overflow" TEST:
T{ 2147483647 2 2 */ -> 2147483647 }T

\ =============================================================================
\ UNSIGNED LESS THAN (U<) TESTS
\ =============================================================================

S" U< with positive numbers" TEST:
T{ 5 10 U< -> -1 }T

S" U< equal numbers" TEST:
T{ 10 10 U< -> 0 }T

S" U< greater number" TEST:
T{ 20 10 U< -> 0 }T

S" U< treats negative as large unsigned" TEST:
T{ -1 0 U< -> 0 }T

S" U< positive vs negative" TEST:
T{ 5 -1 U< -> -1 }T

S" U< both negative" TEST:
T{ -2 -1 U< -> -1 }T

S" U< negative less negative" TEST:
T{ -10 -5 U< -> -1 }T

S" U< max unsigned values" TEST:
T{ 0 -1 U< -> -1 }T

\ =============================================================================
\ BASE TESTS
\ =============================================================================

S" BASE returns address" TEST:
T{ BASE @ 10 = -> -1 }T

S" DECIMAL sets base to 10" TEST:
T{ DECIMAL BASE @ -> 10 }T

S" HEX sets base to 16" TEST:
T{ HEX BASE @ -> 16 }T

S" BINARY sets base to 2" TEST:
T{ BINARY BASE @ -> 2 }T

S" BASE can be modified" TEST:
T{ 8 BASE ! BASE @ -> 8 }T

S" Reset to decimal" TEST:
T{ DECIMAL BASE @ -> 10 }T

\ =============================================================================
\ WITHIN TESTS (using WITHIN from core.fth)
\ =============================================================================

S" WITHIN in range" TEST:
T{ 5 3 10 WITHIN -> -1 }T

S" WITHIN at lower bound" TEST:
T{ 3 3 10 WITHIN -> -1 }T

S" WITHIN at upper bound" TEST:
T{ 10 3 10 WITHIN -> 0 }T

S" WITHIN below range" TEST:
T{ 2 3 10 WITHIN -> 0 }T

S" WITHIN above range" TEST:
T{ 15 3 10 WITHIN -> 0 }T

S" WITHIN negative range" TEST:
T{ -5 -10 0 WITHIN -> -1 }T

S" WITHIN inverted bounds" TEST:
T{ 5 10 3 WITHIN -> 0 }T

S" WITHIN single value range" TEST:
T{ 5 5 6 WITHIN -> -1 }T

S" WITHIN single value at bound" TEST:
T{ 5 5 5 WITHIN -> 0 }T

S" WITHIN zero in positive range" TEST:
T{ 0 -5 5 WITHIN -> -1 }T

S" WITHIN wraparound unsigned" TEST:
T{ 0 -1 10 WITHIN -> -1 }T

S" WITHIN large numbers" TEST:
T{ 1000000 0 2000000 WITHIN -> -1 }T

S" WITHIN negative numbers" TEST:
T{ -50 -100 -10 WITHIN -> -1 }T

S" WITHIN edge of negative" TEST:
T{ -10 -100 -10 WITHIN -> 0 }T

S" WITHIN full range" TEST:
T{ 0 -2147483648 2147483647 WITHIN -> -1 }T

\ =============================================================================
\ ?DUP TESTS
\ =============================================================================

S" ?DUP with zero" TEST:
T{ 0 ?DUP -> 0 }T

S" ?DUP with positive" TEST:
T{ 5 ?DUP -> 5 5 }T

S" ?DUP with negative" TEST:
T{ -3 ?DUP -> -3 -3 }T

S" ?DUP with one" TEST:
T{ 1 ?DUP -> 1 1 }T

S" ?DUP with minus one" TEST:
T{ -1 ?DUP -> -1 -1 }T

S" ?DUP preserves depth with zero" TEST:
T{ 10 20 0 ?DUP -> 10 20 0 }T

S" ?DUP increases depth with nonzero" TEST:
T{ 10 20 5 ?DUP -> 10 20 5 5 }T

\ =============================================================================
\ OUTPUT PRIMITIVES TESTS
\ =============================================================================

S" U. prints unsigned and pops" TEST:
T{ 42 U. DEPTH -> 0 }T

S" .R prints right-justified and pops both" TEST:
T{ 100 10 .R DEPTH -> 0 }T

S" U.R prints unsigned right-justified and pops both" TEST:
T{ 42 8 U.R DEPTH -> 0 }T

S" .S is non-destructive" TEST:
T{ 1 2 3 .S DEPTH -> 1 2 3 3 }T

S" .S shows empty stack" TEST:
T{ .S DEPTH -> 0 }T

\ =============================================================================
\ METAPROGRAMMING TESTS (ISSUE #89)
\ =============================================================================

S" CHAR A returns ASCII 65" TEST:
T{ CHAR A -> 65 }T

S" CHAR Z returns ASCII 90" TEST:
T{ CHAR Z -> 90 }T

S" CHAR 0 returns ASCII 48" TEST:
T{ CHAR 0 -> 48 }T

S" CHAR of multi-char word gets first char" TEST:
T{ CHAR HELLO -> 72 }T

S" [CHAR] in definition" TEST:
: TEST-BRACKET-CHAR-A [CHAR] A ;
T{ TEST-BRACKET-CHAR-A -> 65 }T

S" [CHAR] with SPACE keyword" TEST:
: TEST-BRACKET-CHAR-SPACE [CHAR] SPACE ;
T{ TEST-BRACKET-CHAR-SPACE -> 83 }T

S" ' (TICK) returns xt for DUP" TEST:
T{ 5 ' DUP EXECUTE -> 5 5 }T

S" EXECUTE with +" TEST:
T{ 3 4 ' + EXECUTE -> 7 }T

S" EXECUTE with *" TEST:
T{ 5 6 ' * EXECUTE -> 30 }T

S" ['] in definition with DUP" TEST:
: TEST-TICK-DUP ['] DUP EXECUTE ;
T{ 10 TEST-TICK-DUP -> 10 10 }T

S" ['] in definition with +" TEST:
: TEST-TICK-ADD ['] + EXECUTE ;
T{ 3 4 TEST-TICK-ADD -> 7 }T

S" ['] in definition with SWAP" TEST:
: TEST-TICK-SWAP ['] SWAP EXECUTE ;
T{ 1 2 TEST-TICK-SWAP -> 2 1 }T

S" COUNT converts counted string" TEST:
2 500000 C!
72 500001 C!
73 500002 C!
T{ 500000 COUNT -> 500001 2 }T

S" COUNT - verify character data" TEST:
2 500100 C!
72 500101 C!
73 500102 C!
500100 COUNT DROP
T{ C@ -> 72 }T

S" EXECUTE with user-defined SQUARE" TEST:
: SQUARE DUP * ;
T{ 5 ' SQUARE EXECUTE -> 25 }T

S" EXECUTE with user-defined DOUBLE" TEST:
: DOUBLE 2 * ;
T{ 7 ' DOUBLE EXECUTE -> 14 }T

S" Passing xt as parameter - APPLY-TWICE with DOUBLE" TEST:
: APPLY-TWICE ( n xt -- n' ) DUP >R EXECUTE R> EXECUTE ;
T{ 3 ' DOUBLE APPLY-TWICE -> 12 }T

S" Passing xt as parameter - APPLY-TWICE with SQUARE" TEST:
T{ 4 ' SQUARE APPLY-TWICE -> 256 }T

\ =============================================================================
\ IMMEDIATE AND FIND TESTS
\ =============================================================================

\ Define a word and mark it as immediate
: IMMED-TEST 99 ;
IMMEDIATE

S" FIND with non-immediate word DUP" TEST:
\ Create counted string "DUP" at 500000
3 500000 C!
68 500001 C! 85 500002 C! 80 500003 C!
T{ 500000 FIND SWAP DROP -> -1 }T

S" FIND with immediate word" TEST:
\ Create counted string "IMMED-TEST" at 500100
10 500100 C!
73 500101 C! 77 500102 C! 77 500103 C! 69 500104 C! 68 500105 C!
45 500106 C! 84 500107 C! 69 500108 C! 83 500109 C! 84 500110 C!
T{ 500100 FIND SWAP DROP -> 1 }T

S" FIND with non-existent word" TEST:
\ Create counted string "NOTFOUND" at 500200
8 500200 C!
78 500201 C! 79 500202 C! 84 500203 C! 70 500204 C!
79 500205 C! 85 500206 C! 78 500207 C! 68 500208 C!
T{ 500200 FIND -> 500200 0 }T

S" IMMEDIATE marks last defined word" TEST:
: TEST-IMM 123 ;
IMMEDIATE
8 500300 C!
84 500301 C! 69 500302 C! 83 500303 C! 84 500304 C! 45 500305 C! 73 500306 C! 77 500307 C! 77 500308 C!
T{ 500300 FIND SWAP DROP -> 1 }T

\ =============================================================================
\ ALIGNED, ALIGN, AND FILL TESTS
\ =============================================================================

S" ALIGNED with already aligned address" TEST:
T{ 0 ALIGNED -> 0 }T
T{ 8 ALIGNED -> 8 }T
T{ 16 ALIGNED -> 16 }T
T{ 800 ALIGNED -> 800 }T

S" ALIGNED rounds up to next 8-byte boundary" TEST:
T{ 1 ALIGNED -> 8 }T
T{ 5 ALIGNED -> 8 }T
T{ 7 ALIGNED -> 8 }T
T{ 9 ALIGNED -> 16 }T
T{ 15 ALIGNED -> 16 }T
T{ 17 ALIGNED -> 24 }T

S" ALIGN makes HERE aligned" TEST:
1 ALLOT  \ Make HERE unaligned
ALIGN
T{ HERE 7 AND -> 0 }T  \ Bottom 3 bits should be 0

S" FILL fills memory region with character" TEST:
\ Fill 5 bytes at 600000 with character 65 ('A')
600000 5 65 FILL
T{ 600000 C@ -> 65 }T
T{ 600001 C@ -> 65 }T
T{ 600002 C@ -> 65 }T
T{ 600003 C@ -> 65 }T
T{ 600004 C@ -> 65 }T

S" FILL with different character" TEST:
\ Fill 3 bytes at 600100 with character 88 ('X')
600100 3 88 FILL
T{ 600100 C@ -> 88 }T
T{ 600101 C@ -> 88 }T
T{ 600102 C@ -> 88 }T

S" FILL with zero bytes does nothing" TEST:
\ Store a value, then FILL 0 bytes shouldn't change it
99 600200 C!
600200 0 65 FILL
T{ 600200 C@ -> 99 }T

S" FILL overwrites existing data" TEST:
\ Store some values
10 600300 C!
20 600301 C!
30 600302 C!
\ Fill with 42
600300 3 42 FILL
T{ 600300 C@ -> 42 }T
T{ 600301 C@ -> 42 }T
T{ 600302 C@ -> 42 }T

\ =============================================================================
\ ABORT" TESTS (non-aborting cases only)
\ =============================================================================

S" ABORT-QUOTE with false flag does not abort" TEST:
: TEST-ABORT-FALSE 0 ABORT" This should not print" 42 ;
T{ TEST-ABORT-FALSE -> 42 }T

S" ABORT-QUOTE with false flag in conditional" TEST:
: TEST-COND-ABORT-FALSE
  DUP 0= ABORT" Should not see this"
  ;
T{ 5 TEST-COND-ABORT-FALSE -> 5 }T

\ Note: Cannot test ABORT or ABORT" with true flag in test framework
\ as they would exit the test runner. See /tmp/test_abort*.fth for
\ manual verification tests.

\ =============================================================================
\ >NUMBER TESTS
\ =============================================================================

S" >NUMBER converts decimal string" TEST:
\ Store "123" at address 600000
3 600000 C!
49 600001 C!  \ '1'
50 600002 C!  \ '2'
51 600003 C!  \ '3'
T{ 0 0 600001 3 >NUMBER -> 123 0 600004 0 }T

S" >NUMBER converts hex string" TEST:
\ Store "FF" at address 600100
2 600100 C!
70 600101 C!  \ 'F'
70 600102 C!  \ 'F'
HEX
T{ 0 0 600101 2 >NUMBER -> 255 0 600103 0 }T
DECIMAL

S" >NUMBER stops at invalid character" TEST:
\ Store "12X45" at address 600200
5 600200 C!
49 600201 C!  \ '1'
50 600202 C!  \ '2'
88 600203 C!  \ 'X'
52 600204 C!  \ '4'
53 600205 C!  \ '5'
T{ 0 0 600201 5 >NUMBER -> 12 0 600203 3 }T

S" >NUMBER accumulates to existing value" TEST:
\ Store "99" at address 600300
2 600300 C!
57 600301 C!  \ '9'
57 600302 C!  \ '9'
\ Start with 100 in accumulator, add 99
T{ 100 0 600301 2 >NUMBER -> 10099 0 600303 0 }T

\ =============================================================================
\ STRING TESTS
\ =============================================================================
S" BLANK fills with spaces " TEST:
T{ CREATE BUF 5 ALLOT -> }T
T{ BUF 5 BLANK -> }T
T{ BUF C@ -> 32 }T
T{ BUF 1+ C@ -> 32 }T
T{ BUF 4 + C@ -> 32 }T

S" ERASE fills with zeros" TEST:
T{ CREATE BUF2 5 ALLOT -> }T
T{ 65 BUF2 C! 66 BUF2 1+ C! -> }T
T{ BUF2 5 ERASE -> }T
T{ BUF2 C@ -> 0 }T
T{ BUF2 1+ C@ -> 0 }T
T{ BUF2 4 + C@ -> 0 }T

S" /STRING with 0 offset" TEST:
T{ S" ABCDE" 0 /STRING SWAP DROP -> 5 }T

S" /STRING with positive offset" TEST:
T{ S" ABCDE" 2 /STRING SWAP DROP -> 3 }T

S" /STRING advances address" TEST:
T{ S" ABCDE" 2 /STRING DROP C@ -> 67 }T

S" /STRING with length greater than string" TEST:
T{ S" ABC" 10 /STRING SWAP DROP -> 0 }T

S" COMPARE equal strings" TEST:
T{ S" ABC" S" ABC" COMPARE -> 0 }T

S" COMPARE first string less than second" TEST:
T{ S" ABC" S" ABD" COMPARE -> -1 }T

S" COMPARE first string greater than second" TEST:
T{ S" ABD" S" ABC" COMPARE -> 1 }T

S" COMPARE shorter string less than longer" TEST:
T{ S" AB" S" ABC" COMPARE -> -1 }T

S" COMPARE longer string greater than shorter" TEST:
T{ S" ABC" S" AB" COMPARE -> 1 }T

S" COMPARE empty strings" TEST:
T{ S" " S" " COMPARE -> 0 }T

S" -TRAILING removes trailing spaces" TEST:
T{ CREATE TRBUF 10 ALLOT -> }T
T{ 65 TRBUF C! 66 TRBUF 1+ C! 67 TRBUF 2 + C! -> }T
T{ 32 TRBUF 3 + C! 32 TRBUF 4 + C! -> }T
T{ TRBUF 5 -TRAILING SWAP DROP 3 = -> TRUE }T

S" -TRAILING no trailing spaces" TEST:
T{ S" ABC" -TRAILING SWAP DROP 3 = -> TRUE }T

S" -TRAILING all spaces" TEST:
T{ CREATE SPBUF 5 ALLOT -> }T
T{ SPBUF 5 BLANK -> }T
T{ SPBUF 5 -TRAILING SWAP DROP 0 = -> TRUE }T

S" -TRAILING empty string" TEST:
T{ S" " -TRAILING SWAP DROP 0 = -> TRUE }T

S" SEARCH finds substring at start" TEST:
T{ S" ABCDEF" S" ABC" SEARCH ROT DROP -> 6 TRUE }T

S" SEARCH finds substring in middle" TEST:
T{ S" ABCDEF" S" CDE" SEARCH >R DROP C@ R> -> 67 TRUE }T

S" SEARCH finds substring at end" TEST:
T{ S" ABCDEF" S" DEF" SEARCH ROT DROP -> 3 TRUE }T

S" SEARCH substring not found" TEST:
T{ S" ABCDEF" S" XYZ" SEARCH ROT DROP -> 6 FALSE }T

S" SEARCH empty substring always found" TEST:
T{ S" ABC" S" " SEARCH ROT DROP -> 3 TRUE }T

S" SEARCH in empty string" TEST:
T{ S" " S" A" SEARCH ROT DROP -> 0 FALSE }T

S" C\" creates null-terminated string" TEST:
T{ : TEST-CSTRING C" Hello" ;  -> }T
T{ TEST-CSTRING C@ 72 = -> TRUE }T
T{ TEST-CSTRING 5 + C@ 0 = -> TRUE }T

S" C\" string can be printed character by character" TEST:
: COUNT-CSTRING ( addr -- n )
  0 SWAP
  BEGIN
    DUP C@ DUP
  WHILE
    DROP 1+ SWAP 1+ SWAP
  REPEAT
  2DROP ;
T{ C" Test" COUNT-CSTRING -> 4 }T

\ =============================================================================
\ REPORT
\ =============================================================================

REPORT
