\ REPL Multi-Line Definition Tests
\
\ Tests that multi-line definitions work correctly in the REPL
\ Focuses on core functionality without edge cases
\
\ Run with: ./target/debug/quarter tests/repl_multiline_tests.fth

\ S" stdlib/core.fth" INCLUDED
S" stdlib/test-framework.fth" INCLUDED


TESTING

\ =============================================================================
\ BASIC MULTI-LINE DEFINITIONS
\ =============================================================================

S" Simple multi-line word" TEST:
: SQUARE
  DUP *
;
T{ 5 SQUARE -> 25 }T

S" Multi-line with operations" TEST:
: TRIPLE
  3 *
;
T{ 7 TRIPLE -> 21 }T

S" Multi-line formula" TEST:
: AVERAGE
  + 2 /
;
T{ 10 6 AVERAGE -> 8 }T

\ =============================================================================
\ CONDITIONALS IN MULTI-LINE
\ =============================================================================

S" Single branch conditional" TEST:
: ABS-VALUE
  DUP 0 <
  IF
    NEGATE
  THEN
;
T{ -5 ABS-VALUE -> 5 }T
T{ 5 ABS-VALUE -> 5 }T

S" Two branch conditional" TEST:
: SIGN-ADJUST
  DUP 0 >
  IF
    10 +
  ELSE
    10 -
  THEN
;
T{ 5 SIGN-ADJUST -> 15 }T
T{ -5 SIGN-ADJUST -> -15 }T

\ =============================================================================
\ LOOPS IN MULTI-LINE
\ =============================================================================

S" Post-condition loop" TEST:
: COUNT-UP
  0
  BEGIN
    1 +
    DUP 5 =
  UNTIL
;
T{ COUNT-UP -> 5 }T

S" Counted loop" TEST:
: SUM-RANGE
  0
  10 0 DO
    I +
  LOOP
;
T{ SUM-RANGE -> 45 }T

S" Loop with early exit" TEST:
: FIND-5
  0
  10 0 DO
    I 5 = IF LEAVE THEN
    1 +
  LOOP
;
T{ FIND-5 -> 5 }T

\ =============================================================================
\ RECURSION
\ =============================================================================

S" Recursive factorial" TEST:
: FACTORIAL
  DUP 0=
  IF
    DROP 1
  ELSE
    DUP 1 - FACTORIAL *
  THEN
;
T{ 5 FACTORIAL -> 120 }T
T{ 0 FACTORIAL -> 1 }T

\ =============================================================================
\ COMMENTS IN DEFINITIONS
\ =============================================================================

S" Backslash comments" TEST:
: WITH-COMMENT
  5 \ This is a comment
  DUP +
;
T{ WITH-COMMENT -> 10 }T

S" Parenthesis comments" TEST:
: WITH-PAREN
  5 ( inline comment )
  DUP +
;
T{ WITH-PAREN -> 10 }T

\ =============================================================================
\ NEW PRIMITIVES
\ =============================================================================

S" Star-slash operation" TEST:
: TEST-STAR-SLASH
  6 7 2 */
;
T{ TEST-STAR-SLASH -> 21 }T

S" Unsigned comparison" TEST:
: TEST-U-LESS
  5 10 U<
;
T{ TEST-U-LESS -> -1 }T

S" Question-dup" TEST:
: TEST-QDUP
  5 ?DUP +
;
T{ TEST-QDUP -> 10 }T

S" Variable operations" TEST:
VARIABLE MY-VAR
: TEST-VAR
  42 MY-VAR !
  MY-VAR @
;
T{ TEST-VAR -> 42 }T

S" Base operations" TEST:
: TEST-BASE
  HEX
  BASE @
  DECIMAL
;
T{ TEST-BASE -> 16 }T

S" Within range check" TEST:
: TEST-WITHIN
  5 3 10 WITHIN
;
T{ TEST-WITHIN -> -1 }T

S" Early return" TEST:
: TEST-EXIT
  5
  TRUE IF EXIT THEN
  10
;
T{ TEST-EXIT -> 5 }T

\ =============================================================================
\ REPORT
\ =============================================================================

REPORT
