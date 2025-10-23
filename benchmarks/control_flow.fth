\ Control Flow Benchmarks
\ These benchmarks use control flow (NOT currently JIT-compilable)

\ Benchmark 1: Loop with addition (NOT JIT-compilable - has DO/LOOP)
: LOOP-ADD 0 100 0 DO I + LOOP ;

\ Benchmark 2: Loop with squaring (NOT JIT-compilable - has DO/LOOP)
: LOOP-SQUARE 0 50 0 DO I DUP * + LOOP ;

\ Benchmark 3: Countdown (NOT JIT-compilable - has BEGIN/UNTIL)
: COUNTDOWN 100 BEGIN 1 - DUP 0 = UNTIL DROP ;

\ Benchmark 4: Conditional (NOT JIT-compilable - has IF/THEN)
: ABS DUP 0 < IF NEGATE THEN ;

\ Test the words
LOOP-ADD
LOOP-SQUARE
COUNTDOWN
-42 ABS
