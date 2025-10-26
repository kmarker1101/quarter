\ Master test runner for Quarter Forth
\ Usage: ./quarter tests/run-all-tests.fth --jit
\
\ This file loads the test framework once (so it gets JIT compiled),
\ then loads all test files which use the JIT-compiled test framework.

\ Load test framework (will be JIT compiled in --jit mode)
S" stdlib/test-framework.fth" INCLUDED

\ Load all test files (these just execute tests using the framework)
S" tests/basic_tests.fth" INCLUDED
S" tests/comparison_tests.fth" INCLUDED

\ Note: jit_primitives_tests.fth is excluded - it uses raw output, not the test framework
\ Run it separately: ./quarter tests/jit_primitives_tests.fth --jit
