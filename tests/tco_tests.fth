\ Tail Call Optimization (TCO) Tests
\ These tests verify that tail-recursive functions don't overflow the stack
\
\ Usage: ./quarter tests/tco_tests.fth

\ =============================================================================
\ TAIL-RECURSIVE COUNTDOWN
\ Without TCO, this would overflow the stack for large values
\ =============================================================================

: COUNTDOWN ( n -- )
  DUP 0 > IF
    1 -
    COUNTDOWN  \ Tail call - should be optimized
  ELSE
    DROP
  THEN ;

\ =============================================================================
\ TAIL-RECURSIVE SUM
\ Sum from 0 to n using tail recursion
\ =============================================================================

: SUM-HELPER ( n acc -- result )
  OVER 0 = IF
    SWAP DROP  \ Return accumulator
  ELSE
    SWAP       \ acc n
    DUP >R     \ acc n (save n on return stack)
    +          \ acc+n
    R> 1 -     \ acc+n n-1
    SWAP       \ n-1 acc+n
    SUM-HELPER \ Tail call
  THEN ;

: SUM ( n -- sum )
  0 SUM-HELPER ;

\ =============================================================================
\ TAIL-RECURSIVE FACTORIAL
\ Calculate factorial using accumulator pattern
\ =============================================================================

: FACTORIAL-HELPER ( n acc -- result )
  OVER 1 <= IF
    SWAP DROP  \ Return accumulator
  ELSE
    OVER *     \ Multiply n * acc
    SWAP 1 -   \ Decrement n
    SWAP
    FACTORIAL-HELPER  \ Tail call
  THEN ;

: FACTORIAL ( n -- n! )
  1 FACTORIAL-HELPER ;

\ =============================================================================
\ RUN TESTS
\ =============================================================================

: RUN-TCO-TESTS
  ." Testing Tail Call Optimization..." CR CR

  \ Test 1: Large countdown (would stack overflow without TCO)
  ." Test 1: COUNTDOWN 10000" CR
  10000 COUNTDOWN
  ." OK - No stack overflow" CR CR

  \ Test 2: Sum from 1 to 100
  ." Test 2: SUM 100 (should be 5050)" CR
  100 SUM . CR
  ." Expected: 5050" CR CR

  \ Test 3: Factorial of 10
  ." Test 3: FACTORIAL 10 (should be 3628800)" CR
  10 FACTORIAL . CR
  ." Expected: 3628800" CR CR

  \ Test 4: Very large countdown (stress test)
  ." Test 4: COUNTDOWN 100000 (stress test)" CR
  100000 COUNTDOWN
  ." OK - No stack overflow" CR CR

  ." All TCO tests completed successfully!" CR ;

RUN-TCO-TESTS
