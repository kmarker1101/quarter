\ RECURSE Tests
\ These tests verify that RECURSE works correctly in both interpreted and JIT modes
\
\ Usage:
\   ./quarter tests/recurse_tests.fth         (interpreted mode)
\   ./quarter tests/recurse_tests.fth --jit   (JIT mode)

\ =============================================================================
\ NON-TAIL-RECURSIVE FACTORIAL WITH RECURSE
\ Classic recursive factorial using RECURSE word
\ =============================================================================

: FACTORIAL ( n -- n! )
  DUP 1 <= IF
    DROP 1
  ELSE
    DUP 1 - RECURSE *
  THEN ;

\ =============================================================================
\ NON-TAIL-RECURSIVE FIBONACCI WITH RECURSE
\ Classic recursive fibonacci using RECURSE word
\ =============================================================================

: FIBONACCI ( n -- fib(n) )
  DUP 2 < IF
    \ Base case: fib(0)=0, fib(1)=1
  ELSE
    DUP 1 - RECURSE
    SWAP 2 - RECURSE
    +
  THEN ;

\ =============================================================================
\ TAIL-RECURSIVE COUNTDOWN WITH RECURSE
\ Without TCO, this would overflow the stack for large values
\ =============================================================================

: COUNTDOWN ( n -- )
  DUP 0 > IF
    1 -
    RECURSE  \ Tail call - should be optimized
  ELSE
    DROP
  THEN ;

\ =============================================================================
\ TAIL-RECURSIVE SUM WITH RECURSE
\ Sum from 0 to n using tail recursion with accumulator
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
    RECURSE    \ Tail call
  THEN ;

: SUM ( n -- sum )
  0 SUM-HELPER ;

\ =============================================================================
\ TAIL-RECURSIVE FACTORIAL WITH RECURSE
\ Calculate factorial using accumulator pattern
\ =============================================================================

: FACTORIAL-TAIL-HELPER ( n acc -- result )
  OVER 1 <= IF
    SWAP DROP  \ Return accumulator
  ELSE
    OVER *     \ Multiply n * acc
    SWAP 1 -   \ Decrement n
    SWAP
    RECURSE    \ Tail call
  THEN ;

: FACTORIAL-TAIL ( n -- n! )
  1 FACTORIAL-TAIL-HELPER ;

\ =============================================================================
\ RUN TESTS
\ =============================================================================

: RUN-RECURSE-TESTS
  ." Testing RECURSE word..." CR CR

  \ Test 1: Simple factorial (non-tail-recursive)
  ." Test 1: FACTORIAL 5 (should be 120)" CR
  5 FACTORIAL . CR
  ." Expected: 120" CR CR

  \ Test 2: Fibonacci (doubly recursive)
  ." Test 2: FIBONACCI 10 (should be 55)" CR
  10 FIBONACCI . CR
  ." Expected: 55" CR CR

  \ Test 3: Tail-recursive countdown
  ." Test 3: COUNTDOWN 1000" CR
  1000 COUNTDOWN
  ." OK - No stack overflow" CR CR

  \ Test 4: Tail-recursive sum
  ." Test 4: SUM 100 (should be 5050)" CR
  100 SUM . CR
  ." Expected: 5050" CR CR

  \ Test 5: Tail-recursive factorial
  ." Test 5: FACTORIAL-TAIL 10 (should be 3628800)" CR
  10 FACTORIAL-TAIL . CR
  ." Expected: 3628800" CR CR

  \ Test 6: Larger factorial (non-tail-recursive)
  ." Test 6: FACTORIAL 12 (should be 479001600)" CR
  12 FACTORIAL . CR
  ." Expected: 479001600" CR CR

  \ Test 7: Stress test - large countdown
  ." Test 7: COUNTDOWN 10000 (stress test)" CR
  10000 COUNTDOWN
  ." OK - No stack overflow" CR CR

  ." All RECURSE tests completed successfully!" CR ;

RUN-RECURSE-TESTS
