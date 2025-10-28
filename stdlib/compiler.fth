\ Quarter Forth Self-Hosting Compiler
\ Compiles AST to LLVM IR using primitives exposed from Rust

\ =============================================================================
\ CONSTANTS - AST Node Types
\ =============================================================================

1  CONSTANT AST-PUSH-NUMBER
2  CONSTANT AST-CALL-WORD
3  CONSTANT AST-SEQUENCE
4  CONSTANT AST-IF-THEN-ELSE
5  CONSTANT AST-BEGIN-UNTIL
6  CONSTANT AST-BEGIN-WHILE-REPEAT
7  CONSTANT AST-DO-LOOP
8  CONSTANT AST-PRINT-STRING
9  CONSTANT AST-STACK-STRING
10 CONSTANT AST-LEAVE
11 CONSTANT AST-EXIT

\ =============================================================================
\ MEMORY AREAS FOR COMPILER USE
\ =============================================================================

300000 CONSTANT COMPILER-SCRATCH
301000 CONSTANT WORD-NAME-BUFFER

\ =============================================================================
\ COMPILER STATE
\ =============================================================================

VARIABLE CURRENT-CTX
VARIABLE CURRENT-MODULE
VARIABLE CURRENT-BUILDER
VARIABLE CURRENT-FUNCTION
VARIABLE CURRENT-BLOCK

\ Function parameters (set once per function)
VARIABLE PARAM-MEMORY  \ memory pointer parameter
VARIABLE PARAM-SP      \ sp pointer parameter
VARIABLE PARAM-RP      \ rp pointer parameter

\ Tail call optimization support
302000 CONSTANT CURRENT-WORD-NAME  \ Buffer for current word being compiled
VARIABLE CURRENT-WORD-LEN          \ Length of current word name

\ Batch compilation support
VARIABLE BATCH-MODE                \ Flag: 0 = single word, -1 = batch mode
VARIABLE BATCH-JIT                 \ JIT engine handle for batch mode
VARIABLE CURRENT-AST-HANDLE        \ Temporary storage for AST handle during compilation

\ DO/LOOP compilation temporaries
VARIABLE LOOP-BODY-HANDLE          \ AST handle for loop body
VARIABLE LOOP-INCREMENT            \ Loop increment value
VARIABLE LOOP-START-VALUE          \ Start value handle
VARIABLE LOOP-LIMIT-VALUE          \ Limit value handle
VARIABLE LOOP-PRELOOP-BLOCK        \ Pre-loop block handle
VARIABLE LOOP-LOOP-BLOCK           \ Loop body block handle
VARIABLE LOOP-EXIT-BLOCK           \ Exit block handle
VARIABLE LOOP-PHI-NODE             \ PHI node for loop index
VARIABLE LOOP-OUTER-PHI-NODE       \ Outer loop PHI node (for J)
VARIABLE LOOP-END-BLOCK            \ Loop-end block handle

\ IF/THEN/ELSE compilation temporaries
VARIABLE IF-THEN-HANDLE            \ AST handle for then branch
VARIABLE IF-ELSE-HANDLE            \ AST handle for else branch (0 if no else)
VARIABLE IF-COND-VALUE             \ Condition value handle
VARIABLE IF-THEN-BLOCK             \ Then block handle
VARIABLE IF-ELSE-BLOCK             \ Else block handle (if present)
VARIABLE IF-MERGE-BLOCK            \ Merge block handle

\ =============================================================================
\ HELPER FUNCTIONS
\ =============================================================================

\ Compare two strings for equality
\ ( addr1 len1 addr2 len2 -- flag )
: STRING-EQUALS?
    \ Check lengths first
    ROT OVER <> IF
        \ Lengths differ
        DROP DROP DROP FALSE EXIT
    THEN
    \ Lengths match, compare bytes
    \ Stack: ( addr1 addr2 len )
    0 DO
        OVER I + C@
        OVER I + C@
        <> IF
            \ Bytes differ
            DROP DROP FALSE EXIT
        THEN
    LOOP
    DROP DROP TRUE ;

\ =============================================================================
\ WORD NAME MAPPING
\ Convert Forth word names to C primitive names
\ =============================================================================

\ Map word name to primitive name
\ Stores result at COMPILER-SCRATCH, returns length
\ ( name-addr name-len -- prim-addr prim-len )
: MAP-WORD-NAME
    \ Check for special symbol mappings
    DUP 1 = IF
        OVER C@ 43 = IF  \ '+'
            DROP DROP
            \ Write "quarter_add" (12 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            97  COMPILER-SCRATCH  8 + C!
            100 COMPILER-SCRATCH  9 + C!
            100 COMPILER-SCRATCH 10 + C!
            COMPILER-SCRATCH 11 EXIT
        THEN
        OVER C@ 45 = IF  \ '-'
            DROP DROP
            \ Write "quarter_sub" (12 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            115 COMPILER-SCRATCH  8 + C!
            117 COMPILER-SCRATCH  9 + C!
            98  COMPILER-SCRATCH 10 + C!
            COMPILER-SCRATCH 11 EXIT
        THEN
        OVER C@ 42 = IF  \ '*'
            DROP DROP
            \ Write "quarter_mul" (12 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            109 COMPILER-SCRATCH  8 + C!
            117 COMPILER-SCRATCH  9 + C!
            108 COMPILER-SCRATCH 10 + C!
            COMPILER-SCRATCH 11 EXIT
        THEN
        OVER C@ 47 = IF  \ '/'
            DROP DROP
            \ Write "quarter_div" (12 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            100 COMPILER-SCRATCH  8 + C!
            105 COMPILER-SCRATCH  9 + C!
            118 COMPILER-SCRATCH 10 + C!
            COMPILER-SCRATCH 11 EXIT
        THEN
        OVER C@ 33 = IF  \ '!'
            DROP DROP
            \ Write "quarter_store" (14 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            115 COMPILER-SCRATCH  8 + C!
            116 COMPILER-SCRATCH  9 + C!
            111 COMPILER-SCRATCH 10 + C!
            114 COMPILER-SCRATCH 11 + C!
            101 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        OVER C@ 64 = IF  \ '@'
            DROP DROP
            \ Write "quarter_fetch" (14 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            102 COMPILER-SCRATCH  8 + C!
            101 COMPILER-SCRATCH  9 + C!
            116 COMPILER-SCRATCH 10 + C!
            99  COMPILER-SCRATCH 11 + C!
            104 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        OVER C@ 60 = IF  \ '<'
            DROP DROP
            \ Write "quarter_lt" (11 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            108 COMPILER-SCRATCH  8 + C!
            116 COMPILER-SCRATCH  9 + C!
            COMPILER-SCRATCH 10 EXIT
        THEN
        OVER C@ 62 = IF  \ '>'
            DROP DROP
            \ Write "quarter_gt" (11 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            103 COMPILER-SCRATCH  8 + C!
            116 COMPILER-SCRATCH  9 + C!
            COMPILER-SCRATCH 10 EXIT
        THEN
        \ Check for single equals (61) - '='
        OVER C@ 61 = IF
            DROP DROP
            \ Write "quarter_equal" (14 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            101 COMPILER-SCRATCH  8 + C!
            113 COMPILER-SCRATCH  9 + C!
            117 COMPILER-SCRATCH 10 + C!
            97  COMPILER-SCRATCH 11 + C!
            108 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
    THEN

    \ Check for 2-character special words
    DUP 2 = IF
        \ Check for C! (67, 33)
        OVER C@ 67 = 2 PICK 1 + C@ 33 = AND IF
            DROP DROP
            \ Write "quarter_c_store" (16 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            99  COMPILER-SCRATCH  8 + C!
            95  COMPILER-SCRATCH  9 + C!
            115 COMPILER-SCRATCH 10 + C!
            116 COMPILER-SCRATCH 11 + C!
            111 COMPILER-SCRATCH 12 + C!
            114 COMPILER-SCRATCH 13 + C!
            101 COMPILER-SCRATCH 14 + C!
            COMPILER-SCRATCH 15 EXIT
        THEN
        \ Check for C@ (67, 64)
        OVER C@ 67 = 2 PICK 1 + C@ 64 = AND IF
            DROP DROP
            \ Write "quarter_c_fetch" (16 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            99  COMPILER-SCRATCH  8 + C!
            95  COMPILER-SCRATCH  9 + C!
            102 COMPILER-SCRATCH 10 + C!
            101 COMPILER-SCRATCH 11 + C!
            116 COMPILER-SCRATCH 12 + C!
            99  COMPILER-SCRATCH 13 + C!
            104 COMPILER-SCRATCH 14 + C!
            COMPILER-SCRATCH 15 EXIT
        THEN
        \ Check for >R (62, 82)
        OVER C@ 62 = 2 PICK 1 + C@ 82 = AND IF
            DROP DROP
            \ Write "quarter_to_r" (13 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            116 COMPILER-SCRATCH  8 + C!
            111 COMPILER-SCRATCH  9 + C!
            95  COMPILER-SCRATCH 10 + C!
            114 COMPILER-SCRATCH 11 + C!
            COMPILER-SCRATCH 12 EXIT
        THEN
        \ Check for R> (82, 62)
        OVER C@ 82 = 2 PICK 1 + C@ 62 = AND IF
            DROP DROP
            \ Write "quarter_r_from" (15 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            114 COMPILER-SCRATCH  8 + C!
            95  COMPILER-SCRATCH  9 + C!
            102 COMPILER-SCRATCH 10 + C!
            114 COMPILER-SCRATCH 11 + C!
            111 COMPILER-SCRATCH 12 + C!
            109 COMPILER-SCRATCH 13 + C!
            COMPILER-SCRATCH 14 EXIT
        THEN
        \ Check for R@ (82, 64)
        OVER C@ 82 = 2 PICK 1 + C@ 64 = AND IF
            DROP DROP
            \ Write "quarter_r_fetch" (16 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            114 COMPILER-SCRATCH  8 + C!
            95  COMPILER-SCRATCH  9 + C!
            102 COMPILER-SCRATCH 10 + C!
            101 COMPILER-SCRATCH 11 + C!
            116 COMPILER-SCRATCH 12 + C!
            99  COMPILER-SCRATCH 13 + C!
            104 COMPILER-SCRATCH 14 + C!
            COMPILER-SCRATCH 15 EXIT
        THEN
        \ Check for <> (60, 62) - '<', '>'
        OVER C@ 60 = 2 PICK 1 + C@ 62 = AND IF
            DROP DROP
            \ Write "quarter_not_equal" (18 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            110 COMPILER-SCRATCH  8 + C!
            111 COMPILER-SCRATCH  9 + C!
            116 COMPILER-SCRATCH 10 + C!
            95  COMPILER-SCRATCH 11 + C!
            101 COMPILER-SCRATCH 12 + C!
            113 COMPILER-SCRATCH 13 + C!
            117 COMPILER-SCRATCH 14 + C!
            97  COMPILER-SCRATCH 15 + C!
            108 COMPILER-SCRATCH 16 + C!
            COMPILER-SCRATCH 17 EXIT
        THEN
        \ Check for <= (60, 61) - '<', '='
        OVER C@ 60 = 2 PICK 1 + C@ 61 = AND IF
            DROP DROP
            \ Write "quarter_less_equal" (19 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            108 COMPILER-SCRATCH  8 + C!
            101 COMPILER-SCRATCH  9 + C!
            115 COMPILER-SCRATCH 10 + C!
            115 COMPILER-SCRATCH 11 + C!
            95  COMPILER-SCRATCH 12 + C!
            101 COMPILER-SCRATCH 13 + C!
            113 COMPILER-SCRATCH 14 + C!
            117 COMPILER-SCRATCH 15 + C!
            97  COMPILER-SCRATCH 16 + C!
            108 COMPILER-SCRATCH 17 + C!
            COMPILER-SCRATCH 18 EXIT
        THEN
        \ Check for >= (62, 61) - '>', '='
        OVER C@ 62 = 2 PICK 1 + C@ 61 = AND IF
            DROP DROP
            \ Write "quarter_greater_equal" (22 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            103 COMPILER-SCRATCH  8 + C!
            114 COMPILER-SCRATCH  9 + C!
            101 COMPILER-SCRATCH 10 + C!
            97  COMPILER-SCRATCH 11 + C!
            116 COMPILER-SCRATCH 12 + C!
            101 COMPILER-SCRATCH 13 + C!
            114 COMPILER-SCRATCH 14 + C!
            95  COMPILER-SCRATCH 15 + C!
            101 COMPILER-SCRATCH 16 + C!
            113 COMPILER-SCRATCH 17 + C!
            117 COMPILER-SCRATCH 18 + C!
            97  COMPILER-SCRATCH 19 + C!
            108 COMPILER-SCRATCH 20 + C!
            COMPILER-SCRATCH 21 EXIT
        THEN
        \ Check for U. (85, 46) - 'U', '.'
        OVER C@ 85 = 2 PICK 1 + C@ 46 = AND IF
            DROP DROP
            \ Write "quarter_u_dot" (13 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            117 COMPILER-SCRATCH  8 + C!
            95  COMPILER-SCRATCH  9 + C!
            100 COMPILER-SCRATCH 10 + C!
            111 COMPILER-SCRATCH 11 + C!
            116 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        \ Check for .R (46, 82) - '.', 'R'
        OVER C@ 46 = 2 PICK 1 + C@ 82 = AND IF
            DROP DROP
            \ Write "quarter_dot_r" (13 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            100 COMPILER-SCRATCH  8 + C!
            111 COMPILER-SCRATCH  9 + C!
            116 COMPILER-SCRATCH 10 + C!
            95  COMPILER-SCRATCH 11 + C!
            114 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        \ Check for 1+ (49, 43) - '1', '+'
        OVER C@ 49 = 2 PICK 1 + C@ 43 = AND IF
            DROP DROP
            \ Write "quarter_1plus" (13 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            49  COMPILER-SCRATCH  8 + C!
            112 COMPILER-SCRATCH  9 + C!
            108 COMPILER-SCRATCH 10 + C!
            117 COMPILER-SCRATCH 11 + C!
            115 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        \ Check for 1- (49, 45) - '1', '-'
        OVER C@ 49 = 2 PICK 1 + C@ 45 = AND IF
            DROP DROP
            \ Write "quarter_1minus" (14 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            49  COMPILER-SCRATCH  8 + C!
            109 COMPILER-SCRATCH  9 + C!
            105 COMPILER-SCRATCH 10 + C!
            110 COMPILER-SCRATCH 11 + C!
            117 COMPILER-SCRATCH 12 + C!
            115 COMPILER-SCRATCH 13 + C!
            COMPILER-SCRATCH 14 EXIT
        THEN
        \ Check for 2* (50, 42) - '2', '*'
        OVER C@ 50 = 2 PICK 1 + C@ 42 = AND IF
            DROP DROP
            \ Write "quarter_2star" (13 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            50  COMPILER-SCRATCH  8 + C!
            115 COMPILER-SCRATCH  9 + C!
            116 COMPILER-SCRATCH 10 + C!
            97  COMPILER-SCRATCH 11 + C!
            114 COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        \ Check for 2/ (50, 47) - '2', '/'
        OVER C@ 50 = 2 PICK 1 + C@ 47 = AND IF
            DROP DROP
            \ Write "quarter_2slash" (14 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            50  COMPILER-SCRATCH  8 + C!
            115 COMPILER-SCRATCH  9 + C!
            108 COMPILER-SCRATCH 10 + C!
            97  COMPILER-SCRATCH 11 + C!
            115 COMPILER-SCRATCH 12 + C!
            104 COMPILER-SCRATCH 13 + C!
            COMPILER-SCRATCH 14 EXIT
        THEN
    THEN

    \ Check for 3-character special words
    DUP 3 = IF
        \ Check for SP@ (83, 80, 64) - 'S', 'P', '@'
        OVER C@ 83 = 2 PICK 1 + C@ 80 = AND 2 PICK 2 + C@ 64 = AND IF
                DROP DROP
                \ Write "quarter_sp_fetch" (17 chars)
                113 COMPILER-SCRATCH  0 + C!
                117 COMPILER-SCRATCH  1 + C!
                97  COMPILER-SCRATCH  2 + C!
                114 COMPILER-SCRATCH  3 + C!
                116 COMPILER-SCRATCH  4 + C!
                101 COMPILER-SCRATCH  5 + C!
                114 COMPILER-SCRATCH  6 + C!
                95  COMPILER-SCRATCH  7 + C!
                115 COMPILER-SCRATCH  8 + C!
                112 COMPILER-SCRATCH  9 + C!
                95  COMPILER-SCRATCH 10 + C!
                102 COMPILER-SCRATCH 11 + C!
                101 COMPILER-SCRATCH 12 + C!
                116 COMPILER-SCRATCH 13 + C!
                99  COMPILER-SCRATCH 14 + C!
                104 COMPILER-SCRATCH 15 + C!
                COMPILER-SCRATCH 16 EXIT
            THEN
        \ Check for SP! (83, 80, 33) - 'S', 'P', '!'
        OVER C@ 83 = 2 PICK 1 + C@ 80 = AND 2 PICK 2 + C@ 33 = AND IF
                DROP DROP
                \ Write "quarter_sp_store" (17 chars)
                113 COMPILER-SCRATCH  0 + C!
                117 COMPILER-SCRATCH  1 + C!
                97  COMPILER-SCRATCH  2 + C!
                114 COMPILER-SCRATCH  3 + C!
                116 COMPILER-SCRATCH  4 + C!
                101 COMPILER-SCRATCH  5 + C!
                114 COMPILER-SCRATCH  6 + C!
                95  COMPILER-SCRATCH  7 + C!
                115 COMPILER-SCRATCH  8 + C!
                112 COMPILER-SCRATCH  9 + C!
                95  COMPILER-SCRATCH 10 + C!
                115 COMPILER-SCRATCH 11 + C!
                116 COMPILER-SCRATCH 12 + C!
                111 COMPILER-SCRATCH 13 + C!
                114 COMPILER-SCRATCH 14 + C!
                101 COMPILER-SCRATCH 15 + C!
                COMPILER-SCRATCH 16 EXIT
            THEN
        \ Check for RP@ (82, 80, 64) - 'R', 'P', '@'
        OVER C@ 82 = 2 PICK 1 + C@ 80 = AND 2 PICK 2 + C@ 64 = AND IF
                DROP DROP
                \ Write "quarter_rp_fetch" (17 chars)
                113 COMPILER-SCRATCH  0 + C!
                117 COMPILER-SCRATCH  1 + C!
                97  COMPILER-SCRATCH  2 + C!
                114 COMPILER-SCRATCH  3 + C!
                116 COMPILER-SCRATCH  4 + C!
                101 COMPILER-SCRATCH  5 + C!
                114 COMPILER-SCRATCH  6 + C!
                95  COMPILER-SCRATCH  7 + C!
                114 COMPILER-SCRATCH  8 + C!
                112 COMPILER-SCRATCH  9 + C!
                95  COMPILER-SCRATCH 10 + C!
                102 COMPILER-SCRATCH 11 + C!
                101 COMPILER-SCRATCH 12 + C!
                116 COMPILER-SCRATCH 13 + C!
                99  COMPILER-SCRATCH 14 + C!
                104 COMPILER-SCRATCH 15 + C!
                COMPILER-SCRATCH 16 EXIT
            THEN
        \ Check for RP! (82, 80, 33) - 'R', 'P', '!'
        OVER C@ 82 = 2 PICK 1 + C@ 80 = AND 2 PICK 2 + C@ 33 = AND IF
                DROP DROP
                \ Write "quarter_rp_store" (17 chars)
                113 COMPILER-SCRATCH  0 + C!
                117 COMPILER-SCRATCH  1 + C!
                97  COMPILER-SCRATCH  2 + C!
                114 COMPILER-SCRATCH  3 + C!
                116 COMPILER-SCRATCH  4 + C!
                101 COMPILER-SCRATCH  5 + C!
                114 COMPILER-SCRATCH  6 + C!
                95  COMPILER-SCRATCH  7 + C!
                114 COMPILER-SCRATCH  8 + C!
                112 COMPILER-SCRATCH  9 + C!
                95  COMPILER-SCRATCH 10 + C!
                115 COMPILER-SCRATCH 11 + C!
                116 COMPILER-SCRATCH 12 + C!
                111 COMPILER-SCRATCH 13 + C!
                114 COMPILER-SCRATCH 14 + C!
                101 COMPILER-SCRATCH 15 + C!
                COMPILER-SCRATCH 16 EXIT
            THEN
        \ Check for U.R (85, 46, 82) - 'U', '.', 'R'
        OVER C@ 85 = 2 PICK 1 + C@ 46 = AND 2 PICK 2 + C@ 82 = AND IF
                DROP DROP
                \ Write "quarter_u_dot_r" (15 chars)
                113 COMPILER-SCRATCH  0 + C!
                117 COMPILER-SCRATCH  1 + C!
                97  COMPILER-SCRATCH  2 + C!
                114 COMPILER-SCRATCH  3 + C!
                116 COMPILER-SCRATCH  4 + C!
                101 COMPILER-SCRATCH  5 + C!
                114 COMPILER-SCRATCH  6 + C!
                95  COMPILER-SCRATCH  7 + C!
                117 COMPILER-SCRATCH  8 + C!
                95  COMPILER-SCRATCH  9 + C!
                100 COMPILER-SCRATCH 10 + C!
                111 COMPILER-SCRATCH 11 + C!
                116 COMPILER-SCRATCH 12 + C!
                95  COMPILER-SCRATCH 13 + C!
                114 COMPILER-SCRATCH 14 + C!
                COMPILER-SCRATCH 15 EXIT
            THEN
    THEN
    \ Check for single comma (44)
    DUP 1 = IF
        OVER C@ 44 = IF
            DROP DROP
            \ Write "quarter_comma" (14 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            99  COMPILER-SCRATCH  8 + C!
            111 COMPILER-SCRATCH  9 + C!
            109 COMPILER-SCRATCH 10 + C!
            109 COMPILER-SCRATCH 11 + C!
            97  COMPILER-SCRATCH 12 + C!
            COMPILER-SCRATCH 13 EXIT
        THEN
        \ Check for single dot (46) - '.'
        OVER C@ 46 = IF
            DROP DROP
            \ Write "quarter_dot" (12 chars)
            113 COMPILER-SCRATCH  0 + C!
            117 COMPILER-SCRATCH  1 + C!
            97  COMPILER-SCRATCH  2 + C!
            114 COMPILER-SCRATCH  3 + C!
            116 COMPILER-SCRATCH  4 + C!
            101 COMPILER-SCRATCH  5 + C!
            114 COMPILER-SCRATCH  6 + C!
            95  COMPILER-SCRATCH  7 + C!
            100 COMPILER-SCRATCH  8 + C!
            111 COMPILER-SCRATCH  9 + C!
            116 COMPILER-SCRATCH 10 + C!
            COMPILER-SCRATCH 11 EXIT
        THEN
    THEN

    \ For alphanumeric words (DUP, SWAP, DROP, AND, OR, etc.): lowercase + quarter_ prefix
    \ Build "quarter_" (8 chars)
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!

    \ Copy and lowercase the name
    DUP 0 DO
        OVER I + C@  \ Get char (OVER not 2 PICK - DO popped name-len)
        DUP 65 >= OVER 90 <= AND IF  \ If uppercase A-Z
            32 +  \ Convert to lowercase
        THEN
        COMPILER-SCRATCH 8 + I + C!  \ Store
    LOOP
    NIP 8 + COMPILER-SCRATCH SWAP ;

\ =============================================================================
\ LLVM IR STACK OPERATIONS
\ These generate LLVM IR code that manipulates the runtime stack
\ =============================================================================

\ Compile stack PUSH operation in LLVM IR
\ Takes an LLVM value handle and generates IR to push it onto stack
\ ( value-handle -- )
: COMPILE-PUSH
    \ 1. Load current SP value: sp_val = load(PARAM-SP)
    \ LLVM-BUILD-LOAD expects: ( builder ctx ptr bit-width -- value )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-SP @ 64 LLVM-BUILD-LOAD
    \ Stack: ( value-handle sp-val-handle )

    \ 2. GEP to get memory address: addr = memory + sp_val
    \ LLVM-BUILD-GEP expects: ( builder ctx ptr offset -- result-ptr )
    \ We have: ( value sp-val )
    \ We need: ( builder ctx mem-ptr sp-val )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ 3 PICK LLVM-BUILD-GEP
    \ Stack: ( value sp-val addr-handle )

    \ 3. Store value at address
    \ LLVM-BUILD-STORE expects: ( builder value addr -- )
    \ We have: ( value sp-val addr )
    \ We need: ( builder value addr )
    CURRENT-BUILDER @ 3 PICK 2 PICK LLVM-BUILD-STORE
    DROP \ Drop addr-handle
    \ Stack: ( value sp-val )

    \ 4. Create constant 8 (i64 for pointer arithmetic - 8-byte cells)
    CURRENT-CTX @ 8 64 LLVM-BUILD-CONST-INT
    \ Stack: ( value sp-val eight )

    \ 5. Add: new_sp = sp_val + 8
    \ LLVM-BUILD-ADD expects: ( builder lhs rhs -- result )
    CURRENT-BUILDER @ 2 PICK 2 PICK LLVM-BUILD-ADD
    \ Stack: ( value sp-val four new-sp )
    -ROT DROP DROP
    \ Stack: ( value new-sp )

    \ 6. Store new SP back
    \ LLVM-BUILD-STORE expects: ( builder value addr -- )
    CURRENT-BUILDER @ SWAP PARAM-SP @ LLVM-BUILD-STORE
    DROP ; \ Drop value-handle

\ Compile stack POP operation in LLVM IR
\ Generates IR to pop from stack and returns the LLVM value handle
\ ( -- value-handle )
: COMPILE-POP
    \ 1. Load current SP value: sp_val = load(PARAM-SP)
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-SP @ 64 LLVM-BUILD-LOAD

    \ Stack: ( sp-val-handle )

    \ 2. Create constant 8 (i64 for pointer arithmetic - 8-byte cells)
    CURRENT-CTX @ 8 64 LLVM-BUILD-CONST-INT

    \ Stack: ( sp-val-handle eight-handle )

    \ 3. Subtract: new_sp = sp_val - 8
    CURRENT-BUILDER @ 2 PICK 2 PICK LLVM-BUILD-SUB

    \ Stack: ( sp-val-handle eight-handle new-sp-handle )

    \ 4. Store new SP (duplicate new-sp first to keep it)
    DUP CURRENT-BUILDER @ SWAP PARAM-SP @ LLVM-BUILD-STORE

    \ Stack: ( sp-val-handle eight-handle new-sp-handle )
    -ROT DROP DROP \ Drop sp-val and eight

    \ Stack: ( new-sp-handle )

    \ 5. GEP to get address: addr = memory + new_sp
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ 3 PICK LLVM-BUILD-GEP

    \ Stack: ( new-sp-handle addr-handle )
    NIP \ Drop new-sp-handle

    \ Stack: ( addr-handle )

    \ 6. Load value from address (i64 for stack values - 8-byte cells)
    CURRENT-BUILDER @ CURRENT-CTX @ ROT 64 LLVM-BUILD-LOAD

    \ Stack: ( value-handle )
    ;

\ =============================================================================
\ INLINE PRIMITIVE EMITTERS
\ =============================================================================

\ Emit inline multiplication: pop b, pop a, push (a * b)
\ ( -- )
: EMIT-INLINE-MUL
    \ Pop two values from stack
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    \ Stack: ( b-handle a-handle )

    \ Multiply: a * b
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-MUL
    \ Stack: ( result-handle )

    \ Push result back to stack
    COMPILE-PUSH ;

\ Emit inline addition: pop b, pop a, push (a + b)
\ ( -- )
: EMIT-INLINE-ADD
    \ Pop two values from stack
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    \ Stack: ( b-handle a-handle )

    \ Add: a + b
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-ADD
    \ Stack: ( result-handle )

    \ Push result back to stack
    COMPILE-PUSH ;

\ Emit inline subtraction: pop b, pop a, push (a - b)
\ ( -- )
: EMIT-INLINE-SUB
    \ Pop two values from stack
    COMPILE-POP  \ b (second operand)
    COMPILE-POP  \ a (first operand)
    \ Stack: ( b-handle a-handle )

    \ Swap to get correct order for subtraction
    SWAP
    \ Stack: ( a-handle b-handle )

    \ Subtract: a - b
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SUB
    \ Stack: ( result-handle )

    \ Push result back to stack
    COMPILE-PUSH ;

\ Emit inline division: pop b, pop a, push (a / b)
\ ( -- )
: EMIT-INLINE-DIV
    \ Pop two values from stack
    COMPILE-POP  \ b (divisor, second operand)
    COMPILE-POP  \ a (dividend, first operand)
    \ Stack: ( b-handle a-handle )

    \ Swap to get correct order for division
    SWAP
    \ Stack: ( a-handle b-handle )

    \ Divide: a / b
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SDIV
    \ Stack: ( result-handle )

    \ Push result back to stack
    COMPILE-PUSH ;

\ Emit inline modulo: pop b, pop a, push (a MOD b)
\ ( -- )
: EMIT-INLINE-MOD
    \ Pop two values from stack
    COMPILE-POP  \ b (divisor, second operand)
    COMPILE-POP  \ a (dividend, first operand)
    \ Stack: ( b-handle a-handle )

    \ Swap to get correct order for modulo
    SWAP
    \ Stack: ( a-handle b-handle )

    \ Remainder: a MOD b
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SREM
    \ Stack: ( result-handle )

    \ Push result back to stack
    COMPILE-PUSH ;

\ Emit inline /MOD: pop b, pop a, push (a MOD b) (a / b)
\ Stack effect: ( a b -- remainder quotient )
\ ( -- )
: EMIT-INLINE-/MOD
    \ Pop two values from stack
    COMPILE-POP  \ b (divisor, second operand)
    COMPILE-POP  \ a (dividend, first operand)
    \ Stack: ( b-handle a-handle )

    \ Need both operands for two operations - duplicate them
    2DUP
    \ Stack: ( b-handle a-handle b-handle a-handle )

    \ First: compute remainder (a MOD b)
    SWAP  \ Get correct order for SREM
    \ Stack: ( b-handle a-handle a-handle b-handle )
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SREM
    \ Stack: ( b-handle a-handle remainder-handle )

    \ Push remainder to data stack
    COMPILE-PUSH
    \ Stack: ( b-handle a-handle )

    \ Second: compute quotient (a / b)
    SWAP  \ Get correct order for SDIV
    \ Stack: ( a-handle b-handle )
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SDIV
    \ Stack: ( quotient-handle )

    \ Push quotient to data stack
    COMPILE-PUSH ;

\ Emit inline negate: pop a, push (-a)
\ ( -- )
: EMIT-INLINE-NEGATE
    \ Pop value from stack
    COMPILE-POP  \ a
    \ Stack: ( a-handle )

    \ Build constant 0
    CURRENT-CTX @ 0 64 LLVM-BUILD-CONST-INT
    \ Stack: ( a-handle zero-handle )

    \ Swap to get correct order: 0 - a
    SWAP
    \ Stack: ( zero-handle a-handle )

    \ Subtract: 0 - a
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SUB
    \ Stack: ( result-handle )

    \ Push result back to stack
    COMPILE-PUSH ;

\ Emit inline AND: pop b, pop a, push (a AND b)
\ ( -- )
: EMIT-INLINE-AND
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-AND
    COMPILE-PUSH ;

\ Emit inline OR: pop b, pop a, push (a OR b)
\ ( -- )
: EMIT-INLINE-OR
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-OR
    COMPILE-PUSH ;

\ Emit inline XOR: pop b, pop a, push (a XOR b)
\ ( -- )
: EMIT-INLINE-XOR
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-XOR
    COMPILE-PUSH ;

\ Emit inline INVERT: pop a, push (NOT a) via (a XOR -1)
\ ( -- )
: EMIT-INLINE-INVERT
    COMPILE-POP  \ a
    \ Build constant -1 (all bits set)
    CURRENT-CTX @ -1 64 LLVM-BUILD-CONST-INT
    \ XOR with -1 to flip all bits
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-XOR
    COMPILE-PUSH ;

\ Emit inline LSHIFT: pop b, pop a, push (a << b)
\ ( -- )
: EMIT-INLINE-LSHIFT
    COMPILE-POP  \ b (shift amount, second operand)
    COMPILE-POP  \ a (value to shift, first operand)
    \ Stack: ( b-handle a-handle )

    \ Swap to get correct order for shift
    SWAP
    \ Stack: ( a-handle b-handle )

    \ Shift left: a << b
    CURRENT-BUILDER @ -ROT LLVM-BUILD-SHL
    COMPILE-PUSH ;

\ Emit inline RSHIFT: pop b, pop a, push (a >> b)
\ ( -- )
: EMIT-INLINE-RSHIFT
    COMPILE-POP  \ b (shift amount, second operand)
    COMPILE-POP  \ a (value to shift, first operand)
    \ Stack: ( b-handle a-handle )

    \ Swap to get correct order for shift
    SWAP
    \ Stack: ( a-handle b-handle )

    \ Shift right: a >> b
    CURRENT-BUILDER @ -ROT LLVM-BUILD-ASHR
    COMPILE-PUSH ;

\ Emit inline <: pop b, pop a, push (a < b ? -1 : 0)
\ ( -- )
: EMIT-INLINE-LT
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b (correct order for lhs < rhs)
    CURRENT-BUILDER @ 2       \ a b builder 2
    2SWAP                     \ builder 2 a b
    LLVM-BUILD-ICMP           \ cmp-result (i1)
    \ Convert i1 to i64 (-1 or 0)
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline >: pop b, pop a, push (a > b ? -1 : 0)
\ ( -- )
: EMIT-INLINE-GT
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b
    CURRENT-BUILDER @ 4       \ a b builder 4
    2SWAP                     \ builder 4 a b
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline =: pop b, pop a, push (a = b ? -1 : 0)
\ ( -- )
: EMIT-INLINE-EQ
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b
    CURRENT-BUILDER @ 0       \ a b builder 0
    2SWAP                     \ builder 0 a b
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline <>: pop b, pop a, push (a <> b ? -1 : 0)
\ ( -- )
: EMIT-INLINE-NE
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b
    CURRENT-BUILDER @ 1       \ a b builder 1
    2SWAP                     \ builder 1 a b
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline <=: pop b, pop a, push (a <= b ? -1 : 0)
\ ( -- )
: EMIT-INLINE-LE
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b
    CURRENT-BUILDER @ 3       \ a b builder 3
    2SWAP                     \ builder 3 a b
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline >=: pop b, pop a, push (a >= b ? -1 : 0)
\ ( -- )
: EMIT-INLINE-GE
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b
    CURRENT-BUILDER @ 5       \ a b builder 5
    2SWAP                     \ builder 5 a b
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline U<: pop b, pop a, push (a u< b ? -1 : 0)
\ Unsigned less than comparison using ICMP ULT (predicate 6)
\ ( -- )
: EMIT-INLINE-ULT
    COMPILE-POP COMPILE-POP  \ b a
    SWAP                      \ a b (correct order for lhs < rhs)
    CURRENT-BUILDER @ 6       \ a b builder 6 (ULT predicate)
    2SWAP                     \ builder 6 a b
    LLVM-BUILD-ICMP           \ cmp-result (i1)
    \ Convert i1 to i64 (-1 or 0)
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline 0=: pop a, push (a = 0 ? -1 : 0)
\ ( -- )
: EMIT-INLINE-0EQ
    COMPILE-POP  \ a
    \ Build constant 0
    CURRENT-CTX @ 0 64 LLVM-BUILD-CONST-INT
    \ Stack: ( a-handle zero-handle )
    \ Need: ( builder predicate lhs rhs ) = ( builder 0 a 0 )
    CURRENT-BUILDER @ 0       \ a zero builder 0
    2SWAP                     \ builder 0 a zero
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline 0<: pop a, push (a < 0 ? -1 : 0)
\ ( -- )
: EMIT-INLINE-0LT
    COMPILE-POP  \ a
    \ Build constant 0
    CURRENT-CTX @ 0 64 LLVM-BUILD-CONST-INT
    \ Stack: ( a-handle zero-handle )
    \ Need: ( builder predicate lhs rhs ) = ( builder 2 a 0 )
    CURRENT-BUILDER @ 2       \ a zero builder 2
    2SWAP                     \ builder 2 a zero
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline 0>: pop a, push (a > 0 ? -1 : 0)
\ ( -- )
: EMIT-INLINE-0GT
    COMPILE-POP  \ a
    \ Build constant 0
    CURRENT-CTX @ 0 64 LLVM-BUILD-CONST-INT
    \ Stack: ( a-handle zero-handle )
    \ Need: ( builder predicate lhs rhs ) = ( builder 4 a 0 )
    CURRENT-BUILDER @ 4       \ a zero builder 4
    2SWAP                     \ builder 4 a zero
    LLVM-BUILD-ICMP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    COMPILE-PUSH ;

\ Emit inline DUP: pop a, push a, push a
\ ( -- )
: EMIT-INLINE-DUP
    COMPILE-POP   \ Pop value
    DUP           \ Duplicate the handle
    COMPILE-PUSH  \ Push first copy
    COMPILE-PUSH  \ Push second copy
;

\ Emit inline ?DUP: duplicate if non-zero
\ Stack effect: ( x -- x ) if x=0, ( x -- x x ) if x!=0
\ Implementation: Use basic blocks for conditional duplication
: EMIT-INLINE-?DUP
    \ Pop value from virtual stack
    COMPILE-POP
    \ Stack: ( value-handle )

    \ Save value on return stack for later use
    DUP >R

    \ Compare value to zero (ICMP-NE: not equal, predicate 1)
    CURRENT-BUILDER @ 1 R@
    CURRENT-CTX @ 0 64 LLVM-BUILD-CONST-INT
    LLVM-BUILD-ICMP
    \ Stack: ( cmp-result-handle )

    \ Create "dup" block (for when value != 0)
    CURRENT-CTX @ CURRENT-FUNCTION @
    100 WORD-NAME-BUFFER 0 + C!  \ 'd'
    117 WORD-NAME-BUFFER 1 + C!  \ 'u'
    112 WORD-NAME-BUFFER 2 + C!  \ 'p'
    WORD-NAME-BUFFER 3 LLVM-CREATE-BLOCK
    \ Stack: ( cmp-result dup-block )

    \ Create "skip" block (for when value == 0)
    CURRENT-CTX @ CURRENT-FUNCTION @
    115 WORD-NAME-BUFFER 0 + C!  \ 's'
    107 WORD-NAME-BUFFER 1 + C!  \ 'k'
    105 WORD-NAME-BUFFER 2 + C!  \ 'i'
    112 WORD-NAME-BUFFER 3 + C!  \ 'p'
    WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK
    \ Stack: ( cmp-result dup-block skip-block )

    \ Create "merge" block (where both paths meet)
    CURRENT-CTX @ CURRENT-FUNCTION @
    109 WORD-NAME-BUFFER 0 + C!  \ 'm'
    101 WORD-NAME-BUFFER 1 + C!  \ 'e'
    114 WORD-NAME-BUFFER 2 + C!  \ 'r'
    103 WORD-NAME-BUFFER 3 + C!  \ 'g'
    101 WORD-NAME-BUFFER 4 + C!  \ 'e'
    WORD-NAME-BUFFER 5 LLVM-CREATE-BLOCK
    \ Stack: ( cmp-result dup-block skip-block merge-block )

    \ Save blocks on return stack
    >R >R >R
    \ Stack: ( cmp-result ) R: ( value dup skip merge )

    \ Conditional branch: if (value != 0) goto dup else goto skip
    CURRENT-BUILDER @ SWAP 2 R@ SWAP LLVM-BUILD-COND-BR
    \ Stack: ( ) R: ( value dup skip merge )

    \ Compile "dup" block: push value twice, branch to merge
    R> DUP >R CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
    R@ COMPILE-PUSH
    R@ COMPILE-PUSH
    R> DROP  \ Drop value, get skip block
    R> DUP >R CURRENT-BUILDER @ SWAP LLVM-BUILD-BR
    \ Stack: ( ) R: ( skip merge )

    \ Compile "skip" block: push value once, branch to merge
    R> CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
    R@ COMPILE-PUSH
    R> CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

    \ Position at merge block
    CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
;

\ Emit inline DROP: pop a
\ ( -- )
: EMIT-INLINE-DROP
    COMPILE-POP   \ Pop and discard
    DROP          \ Drop the handle
;

\ Emit inline SWAP: pop b, pop a, push b, push a
\ ( -- )
: EMIT-INLINE-SWAP
    COMPILE-POP   \ b
    COMPILE-POP   \ a
    SWAP          \ Now: b a
    COMPILE-PUSH  \ Push b
    COMPILE-PUSH  \ Push a
;

\ Emit inline OVER: pop x2, pop x1, push x1, push x2, push x1
\ ( -- )
: EMIT-INLINE-OVER
    COMPILE-POP   \ x2 -> stack: ( x2h )
    COMPILE-POP   \ x1 -> stack: ( x2h x1h )
    DUP           \ stack: ( x2h x1h x1h )
    COMPILE-PUSH  \ push x1 -> stack: ( x2h x1h )
    SWAP          \ stack: ( x1h x2h )
    DUP           \ stack: ( x1h x2h x2h )
    COMPILE-PUSH  \ push x2 -> stack: ( x1h x2h )
    DROP          \ stack: ( x1h )
    COMPILE-PUSH  \ push x1
;

\ Emit inline ROT: pop x3, pop x2, pop x1, push x2, push x3, push x1
\ ( -- )
: EMIT-INLINE-ROT
    COMPILE-POP   \ x3 -> stack: ( x3h )
    COMPILE-POP   \ x2 -> stack: ( x3h x2h )
    COMPILE-POP   \ x1 -> stack: ( x3h x2h x1h )
    \ Need to push: x2h, x3h, x1h
    >R            \ Save x1h -> stack: ( x3h x2h ) R: ( x1h )
    SWAP          \ stack: ( x2h x3h )
    DUP           \ stack: ( x2h x3h x3h )
    >R            \ Save x3h -> stack: ( x2h x3h ) R: ( x1h x3h )
    DROP          \ stack: ( x2h )
    DUP           \ stack: ( x2h x2h )
    COMPILE-PUSH  \ push x2 -> stack: ( x2h )
    DROP          \ stack: ( )
    R>            \ Get x3h -> stack: ( x3h )
    DUP           \ stack: ( x3h x3h )
    COMPILE-PUSH  \ push x3 -> stack: ( x3h )
    DROP          \ stack: ( )
    R>            \ Get x1h -> stack: ( x1h )
    COMPILE-PUSH  \ push x1
;

\ Emit inline @ (fetch): pop addr, load i64 from memory[addr], push value
\ ( -- )
: EMIT-INLINE-FETCH
    COMPILE-POP   \ addr -> ( addr )
    \ GEP: ( builder ctx ptr offset -- ptr )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ 3 PICK LLVM-BUILD-GEP
    \ ( addr gep-ptr )
    \ LOAD: ( builder ctx ptr width -- value )
    CURRENT-BUILDER @ CURRENT-CTX @ ROT 64 LLVM-BUILD-LOAD
    \ ( addr value )
    SWAP DROP  \ ( value )
    COMPILE-PUSH
;

\ Emit inline ! (store): pop addr, pop value, store i64 to memory[addr]
\ ( -- )
: EMIT-INLINE-STORE
    COMPILE-POP   \ addr -> ( addr )
    COMPILE-POP   \ value -> ( addr value )
    OVER >R       \ Save addr to R -> ( addr value ) R:( addr )
    \ GEP: ( builder ctx ptr offset -- ptr )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ R> LLVM-BUILD-GEP
    \ ( addr value gep-ptr )
    \ STORE: ( builder value ptr -- )
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-STORE
    \ ( addr )
    DROP
;

\ Emit inline C@ (c-fetch): pop addr, load i8 from memory[addr], sext to i64, push
\ ( -- )
: EMIT-INLINE-C-FETCH
    COMPILE-POP   \ addr -> ( addr )
    \ GEP: ( builder ctx ptr offset -- ptr )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ 3 PICK LLVM-BUILD-GEP
    \ ( addr gep-ptr )
    \ LOAD i8: ( builder ctx ptr width -- value )
    CURRENT-BUILDER @ CURRENT-CTX @ ROT 8 LLVM-BUILD-LOAD
    \ ( addr i8-val )
    \ SEXT i8->i64: ( builder ctx value -- value )
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    \ ( addr i64-val )
    SWAP DROP  \ ( i64-val )
    COMPILE-PUSH
;

\ Emit inline C! (c-store): pop addr, pop value, truncate to i8, store to memory[addr]
\ ( -- )
: EMIT-INLINE-C-STORE
    COMPILE-POP   \ addr -> ( addr )
    COMPILE-POP   \ value -> ( addr value )
    \ TRUNC i64->i8: ( builder ctx value width -- i8-val )
    CURRENT-BUILDER @ CURRENT-CTX @ 2 PICK 8 LLVM-BUILD-TRUNC
    \ ( addr value i8-val )
    ROT >R        \ Save addr -> ( value i8-val ) R:( addr )
    \ GEP: ( builder ctx ptr offset -- ptr )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ R> LLVM-BUILD-GEP
    \ ( value i8-val gep-ptr )
    \ STORE: ( builder value ptr -- )
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-STORE
    \ ( value )
    DROP
;

\ Emit inline +! (add-store): pop addr, pop n, add n to memory[addr]
\ ( -- )
: EMIT-INLINE-ADD-STORE
    COMPILE-POP   \ addr -> ( addr )
    COMPILE-POP   \ n -> ( addr n )
    OVER >R       \ Save addr -> ( addr n ) R:( addr )
    \ GEP: ( builder ctx ptr offset -- ptr )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ R> LLVM-BUILD-GEP
    \ ( addr n gep-ptr )
    DUP >R        \ Save gep-ptr -> ( addr n gep-ptr ) R:( gep-ptr )
    \ LOAD: ( builder ctx ptr width -- old )
    CURRENT-BUILDER @ CURRENT-CTX @ ROT 64 LLVM-BUILD-LOAD
    \ ( addr n old )
    \ ADD: ( builder lhs rhs -- sum )
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-ADD
    \ ( addr sum )
    \ STORE: ( builder value ptr -- )
    R> CURRENT-BUILDER @ ROT ROT LLVM-BUILD-STORE
    \ ( addr )
    DROP
;

\ Emit inline 1+: pop a, push (a + 1)
\ ( -- )
: EMIT-INLINE-1+
    COMPILE-POP  \ a
    \ Build constant 1
    CURRENT-CTX @ 1 64 LLVM-BUILD-CONST-INT
    \ Add: a + 1
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-ADD
    COMPILE-PUSH ;

\ Emit inline 1-: pop a, push (a - 1)
\ ( -- )
: EMIT-INLINE-1-
    COMPILE-POP  \ a
    \ Build constant 1
    CURRENT-CTX @ 1 64 LLVM-BUILD-CONST-INT
    \ Subtract: a - 1
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-SUB
    COMPILE-PUSH ;

\ Emit inline 2*: pop a, push (a * 2) using left shift
\ ( -- )
: EMIT-INLINE-2*
    COMPILE-POP  \ a
    \ Build constant 1 (shift amount)
    CURRENT-CTX @ 1 64 LLVM-BUILD-CONST-INT
    \ Left shift: a << 1
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-SHL
    COMPILE-PUSH ;

\ Emit inline 2/: pop a, push (a / 2) using arithmetic right shift
\ ( -- )
: EMIT-INLINE-2/
    COMPILE-POP  \ a
    \ Build constant 1 (shift amount)
    CURRENT-CTX @ 1 64 LLVM-BUILD-CONST-INT
    \ Arithmetic right shift: a >> 1
    CURRENT-BUILDER @ ROT ROT LLVM-BUILD-ASHR
    COMPILE-PUSH ;

\ Emit inline MIN: pop b, pop a, push (a < b ? a : b)
\ ( -- )
: EMIT-INLINE-MIN
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    \ Stack: ( b a )

    \ Build ICMP: a < b
    \ Need: ( builder predicate lhs rhs ) = ( builder 2 a b )
    CURRENT-BUILDER @ 2 2 PICK 4 PICK LLVM-BUILD-ICMP
    \ Stack: ( b a cond )

    \ Build SELECT: if cond then a else b
    \ Need: ( builder cond true-val false-val ) = ( builder cond a b )
    CURRENT-BUILDER @ SWAP 2 PICK 4 PICK LLVM-BUILD-SELECT
    \ Stack: ( b a result )

    -ROT 2DROP
    COMPILE-PUSH ;

\ Emit inline MAX: pop b, pop a, push (a > b ? a : b)
\ ( -- )
: EMIT-INLINE-MAX
    COMPILE-POP  \ b
    COMPILE-POP  \ a
    \ Stack: ( b a )

    \ Build ICMP: a > b
    \ Need: ( builder predicate lhs rhs ) = ( builder 4 a b )
    CURRENT-BUILDER @ 4 2 PICK 4 PICK LLVM-BUILD-ICMP
    \ Stack: ( b a cond )

    \ Build SELECT: if cond then a else b
    \ Need: ( builder cond true-val false-val ) = ( builder cond a b )
    CURRENT-BUILDER @ SWAP 2 PICK 4 PICK LLVM-BUILD-SELECT
    \ Stack: ( b a result )

    -ROT 2DROP
    COMPILE-PUSH ;

\ Emit inline ABS: pop a, push (a < 0 ? -a : a)
\ ( -- )
: EMIT-INLINE-ABS
    COMPILE-POP  \ a
    \ Stack: ( a )

    \ Build 0 constant
    CURRENT-CTX @ 0 64 LLVM-BUILD-CONST-INT
    \ Stack: ( a 0 )

    \ Build ICMP: a < 0
    \ Need: ( builder predicate lhs rhs ) = ( builder 2 a 0 )
    CURRENT-BUILDER @ 2 3 PICK 3 PICK LLVM-BUILD-ICMP
    \ Stack: ( a 0 cond )

    \ Build -a (0 - a)
    \ Need: ( builder lhs rhs ) = ( builder 0 a )
    CURRENT-BUILDER @ 2 PICK 4 PICK LLVM-BUILD-SUB
    \ Stack: ( a 0 cond -a )

    \ Build SELECT: if cond then -a else a
    \ Need: ( builder cond true-val false-val ) = ( builder cond -a a )
    CURRENT-BUILDER @ 2 PICK 2 PICK 6 PICK LLVM-BUILD-SELECT
    \ Stack: ( a 0 cond -a result )

    -ROT 2DROP NIP NIP
    COMPILE-PUSH ;

\ Emit inline >R: pop from data stack, push to return stack
\ ( -- )
: EMIT-INLINE->R
    \ Step 1: Get value from data stack
    COMPILE-POP  \ Stack: ( value )

    \ Step 2: Load current RP value
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-RP @ 64 LLVM-BUILD-LOAD
    \ Stack: ( value rp )

    \ Step 3: Compute address = memory + rp using GEP
    \ GEP needs: ( builder ctx mem offset )
    CURRENT-BUILDER @  \ ( value rp builder )
    CURRENT-CTX @      \ ( value rp builder ctx )
    PARAM-MEMORY @     \ ( value rp builder ctx mem )
    3 PICK             \ ( value rp builder ctx mem rp ) - rp is at index 3
    LLVM-BUILD-GEP     \ ( value rp addr )

    \ Step 4: Store value at addr
    \ STORE needs: ( builder value ptr )
    CURRENT-BUILDER @  \ ( value rp addr builder )
    3 PICK             \ ( value rp addr builder value )
    2 PICK             \ ( value rp addr builder value addr )
    LLVM-BUILD-STORE   \ ( value rp addr )
    DROP               \ ( value rp )

    \ Step 5: Increment RP by 8
    \ ADD needs: ( builder lhs rhs )
    CURRENT-CTX @ 8 64 LLVM-BUILD-CONST-INT  \ ( value rp eight )
    CURRENT-BUILDER @  \ ( value rp eight builder )
    2 PICK             \ ( value rp eight builder rp )
    2 PICK             \ ( value rp eight builder rp eight )
    LLVM-BUILD-ADD     \ ( value rp eight new-rp )
    NIP NIP            \ ( value new-rp )

    \ Step 6: Store new RP
    \ STORE needs: ( builder value ptr )
    CURRENT-BUILDER @  \ ( value new-rp builder )
    SWAP               \ ( value builder new-rp )
    PARAM-RP @         \ ( value builder new-rp rp-ptr )
    LLVM-BUILD-STORE   \ ( value )
    DROP ;             \ ( )



\ Emit inline R>: pop from return stack, push to data stack
\ ( -- )
: EMIT-INLINE-R>
    \ Step 1: Load current RP
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-RP @ 64 LLVM-BUILD-LOAD
    \ Stack: ( rp )

    \ Step 2: Decrement RP by 8
    \ SUB needs: ( builder lhs rhs )
    CURRENT-CTX @ 8 64 LLVM-BUILD-CONST-INT  \ ( rp eight )
    CURRENT-BUILDER @  \ ( rp eight builder )
    2 PICK             \ ( rp eight builder rp )
    2 PICK             \ ( rp eight builder rp eight )
    LLVM-BUILD-SUB     \ ( rp eight new-rp )
    NIP NIP            \ ( new-rp )

    \ Step 3: Store new RP (keep copy for GEP)
    \ STORE needs: ( builder value ptr )
    DUP                \ ( new-rp new-rp )
    CURRENT-BUILDER @  \ ( new-rp new-rp builder )
    SWAP               \ ( new-rp builder new-rp )
    PARAM-RP @         \ ( new-rp builder new-rp rp-ptr )
    LLVM-BUILD-STORE   \ ( new-rp )

    \ Step 4: GEP to get address = memory + new_rp
    \ GEP needs: ( builder ctx mem offset )
    CURRENT-BUILDER @  \ ( new-rp builder )
    CURRENT-CTX @      \ ( new-rp builder ctx )
    PARAM-MEMORY @     \ ( new-rp builder ctx mem )
    3 PICK             \ ( new-rp builder ctx mem new-rp )
    LLVM-BUILD-GEP     \ ( new-rp addr )

    \ Step 5: Load value from address
    \ LOAD needs: ( builder ctx ptr width )
    CURRENT-BUILDER @  \ ( new-rp addr builder )
    CURRENT-CTX @      \ ( new-rp addr builder ctx )
    2 PICK             \ ( new-rp addr builder ctx addr )
    64                 \ ( new-rp addr builder ctx addr 64 )
    LLVM-BUILD-LOAD    \ ( new-rp addr value )
    NIP NIP            \ ( value )

    \ Step 6: Push to data stack
    COMPILE-PUSH ;

\ Emit inline R@: copy from return stack (peek, don't pop)
\ ( -- )
: EMIT-INLINE-R@
    \ Step 1: Load current RP
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-RP @ 64 LLVM-BUILD-LOAD
    \ Stack: ( rp )

    \ Step 2: Compute peek_rp = rp - 8 (don't store, just use for GEP)
    \ SUB needs: ( builder lhs rhs )
    CURRENT-CTX @ 8 64 LLVM-BUILD-CONST-INT  \ ( rp eight )
    CURRENT-BUILDER @  \ ( rp eight builder )
    2 PICK             \ ( rp eight builder rp )
    2 PICK             \ ( rp eight builder rp eight )
    LLVM-BUILD-SUB     \ ( rp eight peek-rp )
    NIP NIP            \ ( peek-rp )

    \ Step 3: GEP to get address = memory + peek_rp
    \ GEP needs: ( builder ctx mem offset )
    CURRENT-BUILDER @  \ ( peek-rp builder )
    CURRENT-CTX @      \ ( peek-rp builder ctx )
    PARAM-MEMORY @     \ ( peek-rp builder ctx mem )
    3 PICK             \ ( peek-rp builder ctx mem peek-rp )
    LLVM-BUILD-GEP     \ ( peek-rp addr )

    \ Step 4: Load value from address
    \ LOAD needs: ( builder ctx ptr width )
    CURRENT-BUILDER @  \ ( peek-rp addr builder )
    CURRENT-CTX @      \ ( peek-rp addr builder ctx )
    2 PICK             \ ( peek-rp addr builder ctx addr )
    64                 \ ( peek-rp addr builder ctx addr 64 )
    LLVM-BUILD-LOAD    \ ( peek-rp addr value )
    NIP NIP            \ ( value )

    \ Step 5: Push to data stack
    COMPILE-PUSH ;

\ Emit inline I: push current loop index to data stack
\ ( -- )
: EMIT-INLINE-I
    \ The loop index is stored in LOOP-PHI-NODE during compilation
    \ It contains the LLVM value handle for the current loop counter (i32)
    \ Sign-extend from i32 to i64 before pushing to stack
    CURRENT-BUILDER @ CURRENT-CTX @ LOOP-PHI-NODE @ LLVM-BUILD-SEXT
    \ Push the extended value to data stack
    COMPILE-PUSH ;

\ Emit inline J: push outer loop index to data stack
\ ( -- )
\ Note: LOOP-OUTER-PHI-NODE contains the outer loop's PHI node handle
: EMIT-INLINE-J
    \ Get outer loop's PHI handle from variable
    LOOP-OUTER-PHI-NODE @
    \ Sign-extend from i32 to i64 before pushing to stack
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-SEXT
    \ Push the extended value to data stack
    COMPILE-PUSH ;

\ Emit inline SP@: push data stack pointer to data stack
\ ( -- )
: EMIT-INLINE-SP@
    \ Load current SP value (64-bit integer)
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-SP @ 64 LLVM-BUILD-LOAD
    \ Push to data stack
    COMPILE-PUSH ;

\ Emit inline SP!: pop from data stack and store as new SP
\ ( -- )
: EMIT-INLINE-SP!
    \ Pop value from data stack (this will be the new SP)
    COMPILE-POP
    \ Store as new SP
    \ STORE needs: ( builder value ptr )
    CURRENT-BUILDER @
    SWAP
    PARAM-SP @
    LLVM-BUILD-STORE
    DROP ;

\ Emit inline RP@: push return stack pointer to data stack
\ ( -- )
: EMIT-INLINE-RP@
    \ Load current RP value (64-bit integer)
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-RP @ 64 LLVM-BUILD-LOAD
    \ Push to data stack
    COMPILE-PUSH ;

\ Emit inline RP!: pop from data stack and store as new RP
\ ( -- )
: EMIT-INLINE-RP!
    \ Pop value from data stack (this will be the new RP)
    COMPILE-POP
    \ Store as new RP
    \ STORE needs: ( builder value ptr )
    CURRENT-BUILDER @
    SWAP
    PARAM-RP @
    LLVM-BUILD-STORE
    DROP ;

\ Emit inline HERE: push dictionary pointer to data stack
\ ( -- )
: EMIT-INLINE-HERE
    \ Dictionary pointer is stored at fixed memory location 0x01FFF8
    \ Create constant for DP address (0x01FFF8 = 131064)
    CURRENT-CTX @ 131064 64 LLVM-BUILD-CONST-INT
    \ Stack: ( offset-constant )

    \ GEP to get the actual address: memory + offset
    \ GEP needs: ( builder ctx base offset -- ptr )
    CURRENT-BUILDER @
    CURRENT-CTX @
    PARAM-MEMORY @
    3 PICK
    LLVM-BUILD-GEP
    \ Stack: ( offset-constant dp-ptr )

    \ Load the dp value from that address
    \ LOAD needs: ( builder ctx ptr width -- value )
    CURRENT-BUILDER @
    CURRENT-CTX @
    2 PICK
    64
    LLVM-BUILD-LOAD
    \ Stack: ( offset-constant dp-ptr dp-value )

    \ Clean up stack and push to data stack
    NIP NIP
    \ Stack: ( dp-value )

    \ Push to data stack
    COMPILE-PUSH ;

\ =============================================================================
\ AST COMPILATION
\ =============================================================================

\ Forward declaration for recursion
: COMPILE-AST-NODE ;

\ Compile DO/LOOP (type 7)
\ Stack: ( ast-handle -- )
: COMPILE-DO-LOOP
    \ Get loop body and increment, store in variables
    DUP AST-LOOP-BODY LOOP-BODY-HANDLE !
    AST-LOOP-INCREMENT LOOP-INCREMENT !

    \ Pop start and limit from runtime stack (i64 values)
    \ Truncate to i32 for loop index comparison
    COMPILE-POP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT 32 LLVM-BUILD-TRUNC
    LOOP-START-VALUE !

    COMPILE-POP
    CURRENT-BUILDER @ CURRENT-CTX @ ROT 32 LLVM-BUILD-TRUNC
    LOOP-LIMIT-VALUE !

    \ Save pre-loop block
    CURRENT-BUILDER @ LLVM-GET-INSERT-BLOCK LOOP-PRELOOP-BLOCK !

    \ Create loop block
    CURRENT-CTX @ CURRENT-FUNCTION @
    100 WORD-NAME-BUFFER 0 + C!  \ 'd'
    111 WORD-NAME-BUFFER 1 + C!  \ 'o'
    95  WORD-NAME-BUFFER 2 + C!  \ '_'
    108 WORD-NAME-BUFFER 3 + C!  \ 'l'
    111 WORD-NAME-BUFFER 4 + C!  \ 'o'
    111 WORD-NAME-BUFFER 5 + C!  \ 'o'
    112 WORD-NAME-BUFFER 6 + C!  \ 'p'
    WORD-NAME-BUFFER 7 LLVM-CREATE-BLOCK LOOP-LOOP-BLOCK !

    \ Create exit block
    CURRENT-CTX @ CURRENT-FUNCTION @
    100 WORD-NAME-BUFFER 0 + C!  \ 'd'
    111 WORD-NAME-BUFFER 1 + C!  \ 'o'
    95  WORD-NAME-BUFFER 2 + C!  \ '_'
    101 WORD-NAME-BUFFER 3 + C!  \ 'e'
    120 WORD-NAME-BUFFER 4 + C!  \ 'x'
    105 WORD-NAME-BUFFER 5 + C!  \ 'i'
    116 WORD-NAME-BUFFER 6 + C!  \ 't'
    WORD-NAME-BUFFER 7 LLVM-CREATE-BLOCK LOOP-EXIT-BLOCK !

    \ Check if we should enter loop: start < limit
    \ Compare: start < limit (SLT=2)
    CURRENT-BUILDER @ 2 LOOP-START-VALUE @ LOOP-LIMIT-VALUE @ LLVM-BUILD-ICMP
    \ Stack: ( cond-value )

    \ Conditional branch from preloop: if start < limit goto loop, else goto exit
    CURRENT-BUILDER @ SWAP LOOP-LOOP-BLOCK @ LOOP-EXIT-BLOCK @ LLVM-BUILD-COND-BR

    \ Position at loop block
    CURRENT-BUILDER @ LOOP-LOOP-BLOCK @ LLVM-POSITION-AT-END

    \ Save current loop's PHI as outer loop's PHI (for nested loops / J word)
    LOOP-PHI-NODE @ LOOP-OUTER-PHI-NODE !

    \ Create PHI for loop index
    CURRENT-BUILDER @ CURRENT-CTX @
    105 WORD-NAME-BUFFER C!  \ 'i'
    WORD-NAME-BUFFER 1 LLVM-BUILD-PHI LOOP-PHI-NODE !

    \ Add incoming edge from preloop (start value)
    LOOP-PHI-NODE @ LOOP-START-VALUE @ LOOP-PRELOOP-BLOCK @ LLVM-PHI-ADD-INCOMING

    \ Save loop state on return stack before recursive compilation
    LOOP-OUTER-PHI-NODE @ >R   \ Save outer PHI for restoration
    LOOP-PHI-NODE @ >R
    LOOP-START-VALUE @ >R
    LOOP-LIMIT-VALUE @ >R
    LOOP-INCREMENT @ >R
    LOOP-LOOP-BLOCK @ >R
    LOOP-EXIT-BLOCK @ >R

    \ Compile loop body (may contain nested loops)
    LOOP-BODY-HANDLE @ COMPILE-AST-NODE

    \ Restore loop state from return stack
    R> LOOP-EXIT-BLOCK !
    R> LOOP-LOOP-BLOCK !
    R> LOOP-INCREMENT !
    R> LOOP-LIMIT-VALUE !
    R> LOOP-START-VALUE !
    R> LOOP-PHI-NODE !
    R> LOOP-OUTER-PHI-NODE !

    \ Get block after body compilation
    CURRENT-BUILDER @ LLVM-GET-INSERT-BLOCK LOOP-END-BLOCK !

    \ Increment: next = phi + increment
    CURRENT-BUILDER @ LOOP-PHI-NODE @
    CURRENT-CTX @ LOOP-INCREMENT @ 32 LLVM-BUILD-CONST-INT
    LLVM-BUILD-ADD
    \ Stack: ( next-value )

    DUP  \ Duplicate for PHI and comparison
    \ Stack: ( next-value next-value )

    \ Compare: next < limit (SLT=2)
    CURRENT-BUILDER @ 2 ROT LOOP-LIMIT-VALUE @ LLVM-BUILD-ICMP
    \ Stack: ( next-value cond-result )

    SWAP  \ Swap for PHI incoming
    \ Stack: ( cond-result next-value )

    \ Add PHI incoming from loop-end (next value)
    LOOP-PHI-NODE @ SWAP LOOP-END-BLOCK @ LLVM-PHI-ADD-INCOMING
    \ Stack: ( cond-result )

    \ Conditional branch: if true goto loop, else goto exit
    CURRENT-BUILDER @ SWAP LOOP-LOOP-BLOCK @ LOOP-EXIT-BLOCK @ LLVM-BUILD-COND-BR

    \ Position at exit
    CURRENT-BUILDER @ LOOP-EXIT-BLOCK @ LLVM-POSITION-AT-END ;

\ Compile BEGIN/UNTIL loop (type 5)
\ Executes body repeatedly until condition on stack is true (non-zero)
\ Stack effect: ( ast-handle -- )
: COMPILE-BEGIN-UNTIL
    \ Get loop body
    AST-LOOP-BODY

    \ Create loop and exit blocks
    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "loop" to WORD-NAME-BUFFER
    108 WORD-NAME-BUFFER 0 + C!  \ 'l'
    111 WORD-NAME-BUFFER 1 + C!  \ 'o'
    111 WORD-NAME-BUFFER 2 + C!  \ 'o'
    112 WORD-NAME-BUFFER 3 + C!  \ 'p'
    WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK

    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "exit" to WORD-NAME-BUFFER
    101 WORD-NAME-BUFFER 0 + C!  \ 'e'
    120 WORD-NAME-BUFFER 1 + C!  \ 'x'
    105 WORD-NAME-BUFFER 2 + C!  \ 'i'
    116 WORD-NAME-BUFFER 3 + C!  \ 't'
    WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK

    \ Stack: ( body-handle loop-block exit-block )

    \ Jump to loop
    2 PICK CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

    \ Position at loop block
    OVER CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END

    \ Compile body
    2 PICK COMPILE-AST-NODE

    \ Pop condition from stack
    COMPILE-POP

    \ Stack: ( body loop exit cond-handle )

    \ Compare to zero (true=non-zero means exit)
    CURRENT-BUILDER @ 1 ROT
    CURRENT-CTX @ 0 32 LLVM-BUILD-CONST-INT
    LLVM-BUILD-ICMP

    \ Stack: ( body loop exit bool-handle )

    \ Conditional branch: if true exit, else loop
    CURRENT-BUILDER @ SWAP 2 PICK 3 PICK LLVM-BUILD-COND-BR

    \ Stack: ( body loop exit )

    \ Position at exit
    CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END

    \ Clean up
    DROP DROP ;

\ Compile IF/THEN/ELSE (type 4)
\ Stack effect: ( ast-handle -- )
: COMPILE-IF-THEN-ELSE
    \ Get branches and store in variables
    DUP AST-IF-THEN IF-THEN-HANDLE !
    AST-IF-ELSE IF-ELSE-HANDLE !

    \ Pop condition from runtime stack
    COMPILE-POP IF-COND-VALUE !

    \ Compare condition to zero (0=false, nonzero=true)
    CURRENT-BUILDER @ 1 IF-COND-VALUE @
    CURRENT-CTX @ 0 32 LLVM-BUILD-CONST-INT
    LLVM-BUILD-ICMP IF-COND-VALUE !

    \ Create then block
    CURRENT-CTX @ CURRENT-FUNCTION @
    116 WORD-NAME-BUFFER 0 + C!  \ 't'
    104 WORD-NAME-BUFFER 1 + C!  \ 'h'
    101 WORD-NAME-BUFFER 2 + C!  \ 'e'
    110 WORD-NAME-BUFFER 3 + C!  \ 'n'
    WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK IF-THEN-BLOCK !

    \ Create merge block
    CURRENT-CTX @ CURRENT-FUNCTION @
    109 WORD-NAME-BUFFER 0 + C!  \ 'm'
    101 WORD-NAME-BUFFER 1 + C!  \ 'e'
    114 WORD-NAME-BUFFER 2 + C!  \ 'r'
    103 WORD-NAME-BUFFER 3 + C!  \ 'g'
    101 WORD-NAME-BUFFER 4 + C!  \ 'e'
    WORD-NAME-BUFFER 5 LLVM-CREATE-BLOCK IF-MERGE-BLOCK !

    \ Check if we have else branch
    IF-ELSE-HANDLE @ 0 = IF
        \ No ELSE: branch to then or merge
        CURRENT-BUILDER @ IF-COND-VALUE @ IF-THEN-BLOCK @ IF-MERGE-BLOCK @ LLVM-BUILD-COND-BR

        \ Save merge block on return stack (will be used after recursion)
        IF-MERGE-BLOCK @ >R

        \ Compile then branch
        CURRENT-BUILDER @ IF-THEN-BLOCK @ LLVM-POSITION-AT-END
        IF-THEN-HANDLE @ COMPILE-AST-NODE

        \ Restore merge block from return stack
        R>

        \ Branch to merge
        DUP CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

        \ Position at merge
        CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
    ELSE
        \ Create else block
        CURRENT-CTX @ CURRENT-FUNCTION @
        101 WORD-NAME-BUFFER 0 + C!  \ 'e'
        108 WORD-NAME-BUFFER 1 + C!  \ 'l'
        115 WORD-NAME-BUFFER 2 + C!  \ 's'
        101 WORD-NAME-BUFFER 3 + C!  \ 'e'
        WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK IF-ELSE-BLOCK !

        \ Branch to then or else
        CURRENT-BUILDER @ IF-COND-VALUE @ IF-THEN-BLOCK @ IF-ELSE-BLOCK @ LLVM-BUILD-COND-BR

        \ Save blocks on return stack (will be used after recursion)
        IF-MERGE-BLOCK @ >R
        IF-ELSE-BLOCK @ >R

        \ Compile then branch
        CURRENT-BUILDER @ IF-THEN-BLOCK @ LLVM-POSITION-AT-END
        IF-THEN-HANDLE @ COMPILE-AST-NODE

        \ Branch to merge (need to access merge block below else block on return stack)
        CURRENT-BUILDER @ R> R@ SWAP >R LLVM-BUILD-BR

        \ Compile else branch
        CURRENT-BUILDER @ R> LLVM-POSITION-AT-END
        IF-ELSE-HANDLE @ COMPILE-AST-NODE

        \ Restore merge block and branch to it
        R>
        DUP CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

        \ Position at merge
        CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
    THEN ;

\ Main compiler - handles all AST node types recursively
\ Redefine the forward-declared COMPILE-AST-NODE
: COMPILE-AST-NODE ( ast-handle -- )
    \ DUP . CR  \ Uncomment to debug
    DUP AST-TYPE

    \ AST-PUSH-NUMBER (type 1)
    DUP 1 = IF
        DROP
        AST-GET-NUMBER
        CURRENT-CTX @ SWAP 64
        LLVM-BUILD-CONST-INT
        COMPILE-PUSH
        EXIT
    THEN

    \ AST-CALL-WORD (type 2)
    DUP 2 = IF
        DROP
        \ Get word name into WORD-NAME-BUFFER
        WORD-NAME-BUFFER AST-GET-WORD
        \ Stack: ( name-len )

        \ Check if it's an inlinable primitive
        \ Store "*" at COMPILER-SCRATCH for comparison
        42 COMPILER-SCRATCH C!  \ ASCII 42 = '*'

        \ Compare with "*"
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            \ It's multiplication - emit inline
            DROP  \ Drop name-len
            EMIT-INLINE-MUL
            EXIT
        THEN

        \ Check for '+'
        43 COMPILER-SCRATCH C!  \ ASCII 43 = '+'
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP  \ Drop name-len
            EMIT-INLINE-ADD
            EXIT
        THEN

        \ Check for '-'
        45 COMPILER-SCRATCH C!  \ ASCII 45 = '-'
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP  \ Drop name-len
            EMIT-INLINE-SUB
            EXIT
        THEN

        \ Check for '/'
        47 COMPILER-SCRATCH C!  \ ASCII 47 = '/'
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP  \ Drop name-len
            EMIT-INLINE-DIV
            EXIT
        THEN

        \ Check for 'MOD'
        77 COMPILER-SCRATCH C!     \ M
        79 COMPILER-SCRATCH 1 + C! \ O
        68 COMPILER-SCRATCH 2 + C! \ D
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP  \ Drop name-len
            EMIT-INLINE-MOD
            EXIT
        THEN

        \ Check for '/MOD'
        47 COMPILER-SCRATCH C!     \ /
        77 COMPILER-SCRATCH 1 + C! \ M
        79 COMPILER-SCRATCH 2 + C! \ O
        68 COMPILER-SCRATCH 3 + C! \ D
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 4 STRING-EQUALS? IF
            DROP  \ Drop name-len
            EMIT-INLINE-/MOD
            EXIT
        THEN

        \ Check for 'NEGATE'
        78  COMPILER-SCRATCH C!     \ N
        69  COMPILER-SCRATCH 1 + C! \ E
        71  COMPILER-SCRATCH 2 + C! \ G
        65  COMPILER-SCRATCH 3 + C! \ A
        84  COMPILER-SCRATCH 4 + C! \ T
        69  COMPILER-SCRATCH 5 + C! \ E
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 6 STRING-EQUALS? IF
            DROP  \ Drop name-len
            EMIT-INLINE-NEGATE
            EXIT
        THEN

        \ Check for 'AND'
        65 COMPILER-SCRATCH C!     \ A
        78 COMPILER-SCRATCH 1 + C! \ N
        68 COMPILER-SCRATCH 2 + C! \ D
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-AND
            EXIT
        THEN

        \ Check for 'OR'
        79 COMPILER-SCRATCH C!     \ O
        82 COMPILER-SCRATCH 1 + C! \ R
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-OR
            EXIT
        THEN

        \ Check for 'XOR'
        88 COMPILER-SCRATCH C!     \ X
        79 COMPILER-SCRATCH 1 + C! \ O
        82 COMPILER-SCRATCH 2 + C! \ R
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-XOR
            EXIT
        THEN

        \ Check for 'INVERT'
        73  COMPILER-SCRATCH C!     \ I
        78  COMPILER-SCRATCH 1 + C! \ N
        86  COMPILER-SCRATCH 2 + C! \ V
        69  COMPILER-SCRATCH 3 + C! \ E
        82  COMPILER-SCRATCH 4 + C! \ R
        84  COMPILER-SCRATCH 5 + C! \ T
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 6 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-INVERT
            EXIT
        THEN

        \ Check for 'LSHIFT'
        76  COMPILER-SCRATCH C!     \ L
        83  COMPILER-SCRATCH 1 + C! \ S
        72  COMPILER-SCRATCH 2 + C! \ H
        73  COMPILER-SCRATCH 3 + C! \ I
        70  COMPILER-SCRATCH 4 + C! \ F
        84  COMPILER-SCRATCH 5 + C! \ T
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 6 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-LSHIFT
            EXIT
        THEN

        \ Check for 'RSHIFT'
        82  COMPILER-SCRATCH C!     \ R
        83  COMPILER-SCRATCH 1 + C! \ S
        72  COMPILER-SCRATCH 2 + C! \ H
        73  COMPILER-SCRATCH 3 + C! \ I
        70  COMPILER-SCRATCH 4 + C! \ F
        84  COMPILER-SCRATCH 5 + C! \ T
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 6 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-RSHIFT
            EXIT
        THEN

        \ Check for '<'
        60 COMPILER-SCRATCH C!  \ <
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-LT
            EXIT
        THEN

        \ Check for '>'
        62 COMPILER-SCRATCH C!  \ >
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-GT
            EXIT
        THEN

        \ Check for '='
        61 COMPILER-SCRATCH C!  \ =
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-EQ
            EXIT
        THEN

        \ Check for 'U<'
        85 COMPILER-SCRATCH C!      \ U
        60 COMPILER-SCRATCH 1 + C!  \ <
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-ULT
            EXIT
        THEN

        \ Check for '<>'
        60 COMPILER-SCRATCH C!      \ <
        62 COMPILER-SCRATCH 1 + C!  \ >
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-NE
            EXIT
        THEN

        \ Check for '<='
        60 COMPILER-SCRATCH C!      \ <
        61 COMPILER-SCRATCH 1 + C!  \ =
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-LE
            EXIT
        THEN

        \ Check for '>='
        62 COMPILER-SCRATCH C!      \ >
        61 COMPILER-SCRATCH 1 + C!  \ =
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-GE
            EXIT
        THEN

        \ Check for '0='
        48 COMPILER-SCRATCH C!      \ 0
        61 COMPILER-SCRATCH 1 + C!  \ =
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-0EQ
            EXIT
        THEN

        \ Check for '0<'
        48 COMPILER-SCRATCH C!      \ 0
        60 COMPILER-SCRATCH 1 + C!  \ <
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-0LT
            EXIT
        THEN

        \ Check for '0>'
        48 COMPILER-SCRATCH C!      \ 0
        62 COMPILER-SCRATCH 1 + C!  \ >
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-0GT
            EXIT
        THEN

        \ Check for 'DUP'
        68 COMPILER-SCRATCH C!      \ D
        85 COMPILER-SCRATCH 1 + C!  \ U
        80 COMPILER-SCRATCH 2 + C!  \ P
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-DUP
            EXIT
        THEN

        \ Check for '?DUP'
        63 COMPILER-SCRATCH C!      \ ?
        68 COMPILER-SCRATCH 1 + C!  \ D
        85 COMPILER-SCRATCH 2 + C!  \ U
        80 COMPILER-SCRATCH 3 + C!  \ P
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 4 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-?DUP
            EXIT
        THEN

        \ Check for 'DROP'
        68 COMPILER-SCRATCH C!      \ D
        82 COMPILER-SCRATCH 1 + C!  \ R
        79 COMPILER-SCRATCH 2 + C!  \ O
        80 COMPILER-SCRATCH 3 + C!  \ P
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 4 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-DROP
            EXIT
        THEN

        \ Check for 'SWAP'
        83 COMPILER-SCRATCH C!      \ S
        87 COMPILER-SCRATCH 1 + C!  \ W
        65 COMPILER-SCRATCH 2 + C!  \ A
        80 COMPILER-SCRATCH 3 + C!  \ P
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 4 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-SWAP
            EXIT
        THEN

        \ Check for 'OVER'
        79 COMPILER-SCRATCH C!      \ O
        86 COMPILER-SCRATCH 1 + C!  \ V
        69 COMPILER-SCRATCH 2 + C!  \ E
        82 COMPILER-SCRATCH 3 + C!  \ R
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 4 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-OVER
            EXIT
        THEN

        \ Check for 'ROT'
        82 COMPILER-SCRATCH C!      \ R
        79 COMPILER-SCRATCH 1 + C!  \ O
        84 COMPILER-SCRATCH 2 + C!  \ T
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-ROT
            EXIT
        THEN

        \ Check for '@' (fetch)
        64 COMPILER-SCRATCH C!      \ @
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-FETCH
            EXIT
        THEN

        \ Check for '!' (store)
        33 COMPILER-SCRATCH C!      \ !
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-STORE
            EXIT
        THEN

        \ Check for 'C@'
        67 COMPILER-SCRATCH C!      \ C
        64 COMPILER-SCRATCH 1 + C!  \ @
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-C-FETCH
            EXIT
        THEN

        \ Check for 'C!'
        67 COMPILER-SCRATCH C!      \ C
        33 COMPILER-SCRATCH 1 + C!  \ !
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-C-STORE
            EXIT
        THEN

        \ Check for '+!'
        43 COMPILER-SCRATCH C!      \ +
        33 COMPILER-SCRATCH 1 + C!  \ !
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-ADD-STORE
            EXIT
        THEN

        \ Check for '1+'
        49 COMPILER-SCRATCH C!      \ 1
        43 COMPILER-SCRATCH 1 + C!  \ +
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-1+
            EXIT
        THEN

        \ Check for '1-'
        49 COMPILER-SCRATCH C!      \ 1
        45 COMPILER-SCRATCH 1 + C!  \ -
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-1-
            EXIT
        THEN

        \ Check for '2*'
        50 COMPILER-SCRATCH C!      \ 2
        42 COMPILER-SCRATCH 1 + C!  \ *
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-2*
            EXIT
        THEN

        \ Check for '2/'
        50 COMPILER-SCRATCH C!      \ 2
        47 COMPILER-SCRATCH 1 + C!  \ /
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-2/
            EXIT
        THEN

        \ Check for 'MIN'
        77 COMPILER-SCRATCH C!      \ M
        73 COMPILER-SCRATCH 1 + C!  \ I
        78 COMPILER-SCRATCH 2 + C!  \ N
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-MIN
            EXIT
        THEN

        \ Check for 'MAX'
        77 COMPILER-SCRATCH C!      \ M
        65 COMPILER-SCRATCH 1 + C!  \ A
        88 COMPILER-SCRATCH 2 + C!  \ X
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-MAX
            EXIT
        THEN

        \ Check for 'ABS'
        65 COMPILER-SCRATCH C!      \ A
        66 COMPILER-SCRATCH 1 + C!  \ B
        83 COMPILER-SCRATCH 2 + C!  \ S
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-ABS
            EXIT
        THEN

        \ Check for '>R'
        62 COMPILER-SCRATCH C!      \ >
        82 COMPILER-SCRATCH 1 + C!  \ R
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE->R
            EXIT
        THEN

        \ Check for 'R>'
        82 COMPILER-SCRATCH C!      \ R
        62 COMPILER-SCRATCH 1 + C!  \ >
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-R>
            EXIT
        THEN

        \ Check for 'R@'
        82 COMPILER-SCRATCH C!      \ R
        64 COMPILER-SCRATCH 1 + C!  \ @
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 2 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-R@
            EXIT
        THEN

        \ Check for 'I'
        73 COMPILER-SCRATCH C!      \ I
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-I
            EXIT
        THEN

        \ Check for 'J'
        74 COMPILER-SCRATCH C!      \ J
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 1 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-J
            EXIT
        THEN

        \ Check for 'SP@'
        83 COMPILER-SCRATCH C!      \ S
        80 COMPILER-SCRATCH 1 + C!  \ P
        64 COMPILER-SCRATCH 2 + C!  \ @
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-SP@
            EXIT
        THEN

        \ Check for 'SP!'
        83 COMPILER-SCRATCH C!      \ S
        80 COMPILER-SCRATCH 1 + C!  \ P
        33 COMPILER-SCRATCH 2 + C!  \ !
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-SP!
            EXIT
        THEN

        \ Check for 'RP@'
        82 COMPILER-SCRATCH C!      \ R
        80 COMPILER-SCRATCH 1 + C!  \ P
        64 COMPILER-SCRATCH 2 + C!  \ @
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-RP@
            EXIT
        THEN

        \ Check for 'RP!'
        82 COMPILER-SCRATCH C!      \ R
        80 COMPILER-SCRATCH 1 + C!  \ P
        33 COMPILER-SCRATCH 2 + C!  \ !
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 3 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-RP!
            EXIT
        THEN

        \ Check for 'HERE'
        72 COMPILER-SCRATCH C!      \ H
        69 COMPILER-SCRATCH 1 + C!  \ E
        82 COMPILER-SCRATCH 2 + C!  \ R
        69 COMPILER-SCRATCH 3 + C!  \ E
        WORD-NAME-BUFFER OVER COMPILER-SCRATCH 4 STRING-EQUALS? IF
            DROP
            EMIT-INLINE-HERE
            EXIT
        THEN

        \ Not an inlinable primitive - do normal function call
        \ Stack: ( name-len )

        \ Check if this is a recursive call
        \ Compare name-len with CURRENT-WORD-LEN
        DUP CURRENT-WORD-LEN @ = IF
            \ Lengths match - compare the actual strings
            TRUE  \ Assume they match
            DUP 0 DO
                WORD-NAME-BUFFER I + C@
                CURRENT-WORD-NAME I + C@
                = 0= IF  \ Not equal
                    DROP FALSE  \ Mismatch found
                    LEAVE
                THEN
            LOOP

            \ If still TRUE, this is a recursive call
            IF
                DROP  \ Drop name-len, we don't need it

                \ Use CURRENT-FUNCTION directly - it's already the function handle
                \ from when we started compiling this word
                CURRENT-BUILDER @
                CURRENT-FUNCTION @
                PARAM-MEMORY @ PARAM-SP @ PARAM-RP @ 3
                0  \ Not a tail call
                LLVM-BUILD-CALL
                EXIT
            THEN
        THEN

        \ Not recursive - try compiled function first: "_fn_WORDNAME"
        \ Stack: ( name-len )
        DUP >R  \ Save name-len to return stack

        \ Build "_fn_" prefix + word name in COMPILER-SCRATCH
        95  COMPILER-SCRATCH  0 + C!  \ _
        102 COMPILER-SCRATCH  1 + C!  \ f
        110 COMPILER-SCRATCH  2 + C!  \ n
        95  COMPILER-SCRATCH  3 + C!  \ _

        \ Copy word name after prefix
        DUP 0 DO
            WORD-NAME-BUFFER I + C@
            COMPILER-SCRATCH 4 I + + C!
        LOOP
        DROP  \ Drop name-len - we're done with it
        \ Stack: ( )

        \ Try to look up "_fn_WORDNAME"
        \ Need: ( module-handle name-addr name-len+4 -- fn-handle )
        CURRENT-MODULE @
        COMPILER-SCRATCH
        R@ 4 +
        LLVM-MODULE-GET-FUNCTION
        \ Stack: ( fn-handle )

        \ Check if lookup succeeded (non-zero)
        DUP 0= IF
            \ Failed - try primitive name
            DROP
            WORD-NAME-BUFFER R@ MAP-WORD-NAME
            \ Stack: ( prim-addr prim-len )
            CURRENT-MODULE @ -ROT
            LLVM-MODULE-GET-FUNCTION
        THEN

        R> DROP  \ Clean up return stack

        \ Stack: ( fn-handle )

        \ Call with (memory, sp, rp) parameters
        CURRENT-BUILDER @ SWAP
        PARAM-MEMORY @ PARAM-SP @ PARAM-RP @ 3
        0  \ Not a tail call (for now - TCO not implemented yet in JIT)
        LLVM-BUILD-CALL

        EXIT
    THEN

    \ AST-SEQUENCE (type 3)
    DUP 3 = IF
        DROP
        DUP AST-SEQ-LENGTH
        0 DO
            DUP I AST-SEQ-CHILD
            COMPILE-AST-NODE
        LOOP
        DROP
        EXIT
    THEN

    \ AST-IF-THEN-ELSE (type 4)
    DUP 4 = IF
        DROP
        COMPILE-IF-THEN-ELSE
        EXIT
    THEN

    \ AST-BEGIN-UNTIL (type 5)
    DUP 5 = IF
        DROP
        COMPILE-BEGIN-UNTIL
        EXIT
    THEN

    \ AST-DO-LOOP (type 7)
    DUP 7 = IF
        DROP
        COMPILE-DO-LOOP
        EXIT
    THEN

    \ AST-EXIT (type 11) - early return
    DUP 11 = IF
        DROP DROP
        CURRENT-BUILDER @ LLVM-BUILD-RET-VOID
        EXIT
    THEN

    \ Unknown type - just drop
    DROP DROP ;

\ =============================================================================
\ PRIMITIVE DECLARATIONS
\ =============================================================================

\ Declare a single primitive function
\ ( name-addr name-len -- )
: DECLARE-PRIMITIVE
    CURRENT-MODULE @ CURRENT-CTX @ 2SWAP
    LLVM-DECLARE-EXTERNAL ;

\ Declare all primitive functions in the module
: DECLARE-ALL-PRIMITIVES
    \ Arithmetic - quarter_add
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    97  COMPILER-SCRATCH 8 + C!
    100 COMPILER-SCRATCH 9 + C!
    100 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_sub
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    115 COMPILER-SCRATCH 8 + C!
    117 COMPILER-SCRATCH 9 + C!
    98  COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_mul
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    109 COMPILER-SCRATCH 8 + C!
    117 COMPILER-SCRATCH 9 + C!
    108 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_div
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    100 COMPILER-SCRATCH 8 + C!
    105 COMPILER-SCRATCH 9 + C!
    118 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Stack - quarter_dup
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    100 COMPILER-SCRATCH 8 + C!
    117 COMPILER-SCRATCH 9 + C!
    112 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Stack - quarter_swap
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    115 COMPILER-SCRATCH 8 + C!
    119 COMPILER-SCRATCH 9 + C!
    97  COMPILER-SCRATCH 10 + C!
    112 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ Stack - quarter_drop
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    100 COMPILER-SCRATCH 8 + C!
    114 COMPILER-SCRATCH 9 + C!
    111 COMPILER-SCRATCH 10 + C!
    112 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ Output - quarter_dot
    113 COMPILER-SCRATCH 0 + C!
    117 COMPILER-SCRATCH 1 + C!
    97  COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C!
    116 COMPILER-SCRATCH 4 + C!
    101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C!
    95  COMPILER-SCRATCH 7 + C!
    100 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C!
    116 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Comparison - quarter_lt
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 108 COMPILER-SCRATCH 8 + C!
    116 COMPILER-SCRATCH 9 + C!
    COMPILER-SCRATCH 10 DECLARE-PRIMITIVE

    \ Comparison - quarter_gt
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 103 COMPILER-SCRATCH 8 + C!
    116 COMPILER-SCRATCH 9 + C!
    COMPILER-SCRATCH 10 DECLARE-PRIMITIVE

    \ Memory - quarter_store
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 115 COMPILER-SCRATCH 8 + C!
    116 COMPILER-SCRATCH 9 + C! 111 COMPILER-SCRATCH 10 + C! 114 COMPILER-SCRATCH 11 + C!
    101 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Memory - quarter_fetch
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 102 COMPILER-SCRATCH 8 + C!
    101 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C! 99 COMPILER-SCRATCH 11 + C!
    104 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Memory - quarter_c_store
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 99 COMPILER-SCRATCH 8 + C!
    95 COMPILER-SCRATCH 9 + C! 115 COMPILER-SCRATCH 10 + C! 116 COMPILER-SCRATCH 11 + C!
    111 COMPILER-SCRATCH 12 + C! 114 COMPILER-SCRATCH 13 + C! 101 COMPILER-SCRATCH 14 + C!
    COMPILER-SCRATCH 15 DECLARE-PRIMITIVE

    \ Memory - quarter_c_fetch
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 99 COMPILER-SCRATCH 8 + C!
    95 COMPILER-SCRATCH 9 + C! 102 COMPILER-SCRATCH 10 + C! 101 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 99 COMPILER-SCRATCH 13 + C! 104 COMPILER-SCRATCH 14 + C!
    COMPILER-SCRATCH 15 DECLARE-PRIMITIVE

    \ Bitwise - quarter_and
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 97 COMPILER-SCRATCH 8 + C!
    110 COMPILER-SCRATCH 9 + C! 100 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Bitwise - quarter_or
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 111 COMPILER-SCRATCH 8 + C!
    114 COMPILER-SCRATCH 9 + C!
    COMPILER-SCRATCH 10 DECLARE-PRIMITIVE

    \ Bitwise - quarter_xor
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 120 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 114 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Bitwise - quarter_invert
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 105 COMPILER-SCRATCH 8 + C!
    110 COMPILER-SCRATCH 9 + C! 118 COMPILER-SCRATCH 10 + C! 101 COMPILER-SCRATCH 11 + C!
    114 COMPILER-SCRATCH 12 + C! 116 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ Bitwise - quarter_lshift
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 108 COMPILER-SCRATCH 8 + C!
    115 COMPILER-SCRATCH 9 + C! 104 COMPILER-SCRATCH 10 + C! 105 COMPILER-SCRATCH 11 + C!
    102 COMPILER-SCRATCH 12 + C! 116 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ Bitwise - quarter_rshift
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 114 COMPILER-SCRATCH 8 + C!
    115 COMPILER-SCRATCH 9 + C! 104 COMPILER-SCRATCH 10 + C! 105 COMPILER-SCRATCH 11 + C!
    102 COMPILER-SCRATCH 12 + C! 116 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ Return stack - quarter_to_r
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 116 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 95 COMPILER-SCRATCH 10 + C! 114 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ Return stack - quarter_r_from
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 114 COMPILER-SCRATCH 8 + C!
    95 COMPILER-SCRATCH 9 + C! 102 COMPILER-SCRATCH 10 + C! 114 COMPILER-SCRATCH 11 + C!
    111 COMPILER-SCRATCH 12 + C! 109 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ Return stack - quarter_r_fetch
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 114 COMPILER-SCRATCH 8 + C!
    95 COMPILER-SCRATCH 9 + C! 102 COMPILER-SCRATCH 10 + C! 101 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 99 COMPILER-SCRATCH 13 + C! 104 COMPILER-SCRATCH 14 + C!
    COMPILER-SCRATCH 15 DECLARE-PRIMITIVE

    \ Stack - quarter_over
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 111 COMPILER-SCRATCH 8 + C!
    118 COMPILER-SCRATCH 9 + C! 101 COMPILER-SCRATCH 10 + C! 114 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ Stack - quarter_rot
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 114 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Stack - quarter_pick
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 112 COMPILER-SCRATCH 8 + C!
    105 COMPILER-SCRATCH 9 + C! 99 COMPILER-SCRATCH 10 + C! 107 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ Stack - quarter_depth
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 100 COMPILER-SCRATCH 8 + C!
    101 COMPILER-SCRATCH 9 + C! 112 COMPILER-SCRATCH 10 + C! 116 COMPILER-SCRATCH 11 + C!
    104 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_slash_mod
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 115 COMPILER-SCRATCH 8 + C!
    108 COMPILER-SCRATCH 9 + C! 97 COMPILER-SCRATCH 10 + C! 115 COMPILER-SCRATCH 11 + C!
    104 COMPILER-SCRATCH 12 + C! 95 COMPILER-SCRATCH 13 + C! 109 COMPILER-SCRATCH 14 + C!
    111 COMPILER-SCRATCH 15 + C! 100 COMPILER-SCRATCH 16 + C!
    COMPILER-SCRATCH 17 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_negate
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 110 COMPILER-SCRATCH 8 + C!
    101 COMPILER-SCRATCH 9 + C! 103 COMPILER-SCRATCH 10 + C! 97 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 101 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_abs
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 97 COMPILER-SCRATCH 8 + C!
    98 COMPILER-SCRATCH 9 + C! 115 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Loop - quarter_i
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 105 COMPILER-SCRATCH 8 + C!
    COMPILER-SCRATCH 9 DECLARE-PRIMITIVE

    \ Loop - quarter_j
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 106 COMPILER-SCRATCH 8 + C!
    COMPILER-SCRATCH 9 DECLARE-PRIMITIVE

    \ I/O - quarter_emit
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 101 COMPILER-SCRATCH 8 + C!
    109 COMPILER-SCRATCH 9 + C! 105 COMPILER-SCRATCH 10 + C! 116 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ I/O - quarter_key
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 107 COMPILER-SCRATCH 8 + C!
    101 COMPILER-SCRATCH 9 + C! 121 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ I/O - quarter_cr
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 99 COMPILER-SCRATCH 8 + C!
    114 COMPILER-SCRATCH 9 + C!
    COMPILER-SCRATCH 10 DECLARE-PRIMITIVE

    \ I/O - quarter_dot
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 100 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Stack pointers - quarter_sp_fetch
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 115 COMPILER-SCRATCH 8 + C!
    112 COMPILER-SCRATCH 9 + C! 95 COMPILER-SCRATCH 10 + C! 102 COMPILER-SCRATCH 11 + C!
    101 COMPILER-SCRATCH 12 + C! 116 COMPILER-SCRATCH 13 + C! 99 COMPILER-SCRATCH 14 + C!
    104 COMPILER-SCRATCH 15 + C!
    COMPILER-SCRATCH 16 DECLARE-PRIMITIVE

    \ Stack pointers - quarter_sp_store
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 115 COMPILER-SCRATCH 8 + C!
    112 COMPILER-SCRATCH 9 + C! 95 COMPILER-SCRATCH 10 + C! 115 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 111 COMPILER-SCRATCH 13 + C! 114 COMPILER-SCRATCH 14 + C!
    101 COMPILER-SCRATCH 15 + C!
    COMPILER-SCRATCH 16 DECLARE-PRIMITIVE

    \ Stack pointers - quarter_rp_fetch
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 114 COMPILER-SCRATCH 8 + C!
    112 COMPILER-SCRATCH 9 + C! 95 COMPILER-SCRATCH 10 + C! 102 COMPILER-SCRATCH 11 + C!
    101 COMPILER-SCRATCH 12 + C! 116 COMPILER-SCRATCH 13 + C! 99 COMPILER-SCRATCH 14 + C!
    104 COMPILER-SCRATCH 15 + C!
    COMPILER-SCRATCH 16 DECLARE-PRIMITIVE

    \ Stack pointers - quarter_rp_store
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 114 COMPILER-SCRATCH 8 + C!
    112 COMPILER-SCRATCH 9 + C! 95 COMPILER-SCRATCH 10 + C! 115 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 111 COMPILER-SCRATCH 13 + C! 114 COMPILER-SCRATCH 14 + C!
    101 COMPILER-SCRATCH 15 + C!
    COMPILER-SCRATCH 16 DECLARE-PRIMITIVE

    \ Memory allocation - quarter_here
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 104 COMPILER-SCRATCH 8 + C!
    101 COMPILER-SCRATCH 9 + C! 114 COMPILER-SCRATCH 10 + C! 101 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ Memory allocation - quarter_allot
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 97 COMPILER-SCRATCH 8 + C!
    108 COMPILER-SCRATCH 9 + C! 108 COMPILER-SCRATCH 10 + C! 111 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Memory allocation - quarter_comma
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 99 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 109 COMPILER-SCRATCH 10 + C! 109 COMPILER-SCRATCH 11 + C!
    97 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Comparison - quarter_equal
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 101 COMPILER-SCRATCH 8 + C!
    113 COMPILER-SCRATCH 9 + C! 117 COMPILER-SCRATCH 10 + C! 97 COMPILER-SCRATCH 11 + C!
    108 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Comparison - quarter_not_equal
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 110 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C! 95 COMPILER-SCRATCH 11 + C!
    101 COMPILER-SCRATCH 12 + C! 113 COMPILER-SCRATCH 13 + C! 117 COMPILER-SCRATCH 14 + C!
    97 COMPILER-SCRATCH 15 + C! 108 COMPILER-SCRATCH 16 + C!
    COMPILER-SCRATCH 17 DECLARE-PRIMITIVE

    \ Comparison - quarter_less_equal
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 108 COMPILER-SCRATCH 8 + C!
    101 COMPILER-SCRATCH 9 + C! 115 COMPILER-SCRATCH 10 + C! 115 COMPILER-SCRATCH 11 + C!
    95 COMPILER-SCRATCH 12 + C! 101 COMPILER-SCRATCH 13 + C! 113 COMPILER-SCRATCH 14 + C!
    117 COMPILER-SCRATCH 15 + C! 97 COMPILER-SCRATCH 16 + C! 108 COMPILER-SCRATCH 17 + C!
    COMPILER-SCRATCH 18 DECLARE-PRIMITIVE

    \ Comparison - quarter_greater_equal
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 103 COMPILER-SCRATCH 8 + C!
    114 COMPILER-SCRATCH 9 + C! 101 COMPILER-SCRATCH 10 + C! 97 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 101 COMPILER-SCRATCH 13 + C! 114 COMPILER-SCRATCH 14 + C!
    95 COMPILER-SCRATCH 15 + C! 101 COMPILER-SCRATCH 16 + C! 113 COMPILER-SCRATCH 17 + C!
    117 COMPILER-SCRATCH 18 + C! 97 COMPILER-SCRATCH 19 + C! 108 COMPILER-SCRATCH 20 + C!
    COMPILER-SCRATCH 21 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_min
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 109 COMPILER-SCRATCH 8 + C!
    105 COMPILER-SCRATCH 9 + C! 110 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_max
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 109 COMPILER-SCRATCH 8 + C!
    97 COMPILER-SCRATCH 9 + C! 120 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_1plus
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 49 COMPILER-SCRATCH 8 + C!
    112 COMPILER-SCRATCH 9 + C! 108 COMPILER-SCRATCH 10 + C! 117 COMPILER-SCRATCH 11 + C!
    115 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_1minus
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 49 COMPILER-SCRATCH 8 + C!
    109 COMPILER-SCRATCH 9 + C! 105 COMPILER-SCRATCH 10 + C! 110 COMPILER-SCRATCH 11 + C!
    117 COMPILER-SCRATCH 12 + C! 115 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_2star
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 50 COMPILER-SCRATCH 8 + C!
    115 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C! 97 COMPILER-SCRATCH 11 + C!
    114 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ Arithmetic - quarter_2slash
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 50 COMPILER-SCRATCH 8 + C!
    115 COMPILER-SCRATCH 9 + C! 108 COMPILER-SCRATCH 10 + C! 97 COMPILER-SCRATCH 11 + C!
    115 COMPILER-SCRATCH 12 + C! 104 COMPILER-SCRATCH 13 + C!
    COMPILER-SCRATCH 14 DECLARE-PRIMITIVE

    \ I/O - quarter_u_dot
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 117 COMPILER-SCRATCH 8 + C!
    95 COMPILER-SCRATCH 9 + C! 100 COMPILER-SCRATCH 10 + C! 111 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ I/O - quarter_dot_r
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 100 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C! 95 COMPILER-SCRATCH 11 + C!
    114 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE

    \ I/O - quarter_u_dot_r
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 117 COMPILER-SCRATCH 8 + C!
    95 COMPILER-SCRATCH 9 + C! 100 COMPILER-SCRATCH 10 + C! 111 COMPILER-SCRATCH 11 + C!
    116 COMPILER-SCRATCH 12 + C! 95 COMPILER-SCRATCH 13 + C! 114 COMPILER-SCRATCH 14 + C!
    COMPILER-SCRATCH 15 DECLARE-PRIMITIVE

    \ I/O - quarter_type
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 116 COMPILER-SCRATCH 8 + C!
    121 COMPILER-SCRATCH 9 + C! 112 COMPILER-SCRATCH 10 + C! 101 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ I/O - quarter_dot
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 100 COMPILER-SCRATCH 8 + C!
    111 COMPILER-SCRATCH 9 + C! 116 COMPILER-SCRATCH 10 + C!
    COMPILER-SCRATCH 11 DECLARE-PRIMITIVE

    \ I/O - quarter_cr
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 99 COMPILER-SCRATCH 8 + C!
    114 COMPILER-SCRATCH 9 + C!
    COMPILER-SCRATCH 10 DECLARE-PRIMITIVE

    \ I/O - quarter_emit
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 101 COMPILER-SCRATCH 8 + C!
    109 COMPILER-SCRATCH 9 + C! 105 COMPILER-SCRATCH 10 + C! 116 COMPILER-SCRATCH 11 + C!
    COMPILER-SCRATCH 12 DECLARE-PRIMITIVE

    \ I/O - quarter_space
    113 COMPILER-SCRATCH 0 + C! 117 COMPILER-SCRATCH 1 + C! 97 COMPILER-SCRATCH 2 + C!
    114 COMPILER-SCRATCH 3 + C! 116 COMPILER-SCRATCH 4 + C! 101 COMPILER-SCRATCH 5 + C!
    114 COMPILER-SCRATCH 6 + C! 95 COMPILER-SCRATCH 7 + C! 115 COMPILER-SCRATCH 8 + C!
    112 COMPILER-SCRATCH 9 + C! 97 COMPILER-SCRATCH 10 + C! 99 COMPILER-SCRATCH 11 + C!
    101 COMPILER-SCRATCH 12 + C!
    COMPILER-SCRATCH 13 DECLARE-PRIMITIVE
;

\ =============================================================================
\ BATCH COMPILATION SUPPORT
\ =============================================================================

\ Initialize batch compilation mode - creates one global module
: INIT-BATCH-COMPILER
    \ Create LLVM context
    LLVM-CREATE-CONTEXT CURRENT-CTX !

    \ Create global module
    CURRENT-CTX @
    \ Write "batch_module" to WORD-NAME-BUFFER
    98  WORD-NAME-BUFFER  0 + C!  \ 'b'
    97  WORD-NAME-BUFFER  1 + C!  \ 'a'
    116 WORD-NAME-BUFFER  2 + C!  \ 't'
    99  WORD-NAME-BUFFER  3 + C!  \ 'c'
    104 WORD-NAME-BUFFER  4 + C!  \ 'h'
    95  WORD-NAME-BUFFER  5 + C!  \ '_'
    109 WORD-NAME-BUFFER  6 + C!  \ 'm'
    111 WORD-NAME-BUFFER  7 + C!  \ 'o'
    100 WORD-NAME-BUFFER  8 + C!  \ 'd'
    117 WORD-NAME-BUFFER  9 + C!  \ 'u'
    108 WORD-NAME-BUFFER 10 + C!  \ 'l'
    101 WORD-NAME-BUFFER 11 + C!  \ 'e'
    WORD-NAME-BUFFER 12 LLVM-CREATE-MODULE CURRENT-MODULE !

    \ Declare all primitive functions
    DECLARE-ALL-PRIMITIVES

    \ Create global builder
    CURRENT-CTX @ LLVM-CREATE-BUILDER CURRENT-BUILDER !

    \ Set batch mode flag
    -1 BATCH-MODE !
;

\ Finalize batch compilation - creates JIT and returns handle
\ Returns JIT engine handle
: FINALIZE-BATCH ( -- jit-handle )
    \ Create JIT execution engine
    CURRENT-MODULE @ LLVM-CREATE-JIT

    \ Clear batch mode flag
    0 BATCH-MODE !
;

\ =============================================================================
\ COMPILER ENTRY POINT
\ =============================================================================

\ Declare a function signature without compiling the body
\ Used in pass 1 of batch compilation to create forward references
\ ( name-addr name-len -- )
: DECLARE-FUNCTION
    \ Copy name to buffer
    DUP CURRENT-WORD-LEN !
    OVER OVER
    0 DO
        DUP I + C@
        CURRENT-WORD-NAME I + C!
    LOOP
    DROP DROP

    \ Build function name: "_fn_WORDNAME"
    95  WORD-NAME-BUFFER 0 + C!  \ _
    102 WORD-NAME-BUFFER 1 + C!  \ f
    110 WORD-NAME-BUFFER 2 + C!  \ n
    95  WORD-NAME-BUFFER 3 + C!  \ _

    \ Copy word name after prefix
    CURRENT-WORD-LEN @ 0 DO
        CURRENT-WORD-NAME I + C@
        WORD-NAME-BUFFER 4 I + + C!
    LOOP

    \ Create function in module
    CURRENT-MODULE @
    CURRENT-CTX @
    WORD-NAME-BUFFER
    CURRENT-WORD-LEN @ 4 +
    LLVM-CREATE-FUNCTION

    \ Don't save the function handle - we'll look it up later
    DROP ;

\ Compile a word from its AST
\ Returns JIT function pointer (or 0 in batch mode - get from JIT later)
: COMPILE-WORD ( ast-handle name-addr name-len -- fn-ptr )
    \ Save name length and copy name to buffer
    DUP CURRENT-WORD-LEN !  \ Store length
    \ Stack: ( ast name-addr name-len )
    OVER OVER  \ ( ast name-addr name-len name-addr name-len )
    0 DO
        DUP I + C@  \ Get byte from source
        CURRENT-WORD-NAME I + C!  \ Store to buffer
    LOOP
    DROP  \ Drop name-addr copy
    \ Stack: ( ast name-addr name-len )

    \ Save name and AST handle before any complex stack operations
    >R >R  \ Save name-addr and name-len on return stack
    \ Stack: ( ast-handle )
    CURRENT-AST-HANDLE !  \ Store AST handle to variable
    \ Stack: ( )

    \ Check if in batch mode
    BATCH-MODE @ IF
        \ In batch mode - use existing global context/module
        \ Skip context and module creation
    ELSE
        \ Not in batch mode - create context and module per word
        \ Create LLVM context
        LLVM-CREATE-CONTEXT CURRENT-CTX !

        \ Create module
        CURRENT-CTX @
        \ Write "module" to WORD-NAME-BUFFER
        109 WORD-NAME-BUFFER 0 + C!  \ 'm'
        111 WORD-NAME-BUFFER 1 + C!  \ 'o'
        100 WORD-NAME-BUFFER 2 + C!  \ 'd'
        117 WORD-NAME-BUFFER 3 + C!  \ 'u'
        108 WORD-NAME-BUFFER 4 + C!  \ 'l'
        101 WORD-NAME-BUFFER 5 + C!  \ 'e'
        WORD-NAME-BUFFER 6 LLVM-CREATE-MODULE CURRENT-MODULE !

        \ Declare all primitive functions
        DECLARE-ALL-PRIMITIVES

        \ Create builder
        CURRENT-CTX @ LLVM-CREATE-BUILDER CURRENT-BUILDER !
    THEN

    \ Get or create function
    BATCH-MODE @ IF
        \ Batch mode: look up function that was declared in pass 1
        95 WORD-NAME-BUFFER 0 + C!   \ '_'
        102 WORD-NAME-BUFFER 1 + C!  \ 'f'
        110 WORD-NAME-BUFFER 2 + C!  \ 'n'
        95 WORD-NAME-BUFFER 3 + C!   \ '_'
        \ Append word name
        CURRENT-WORD-LEN @ 0 DO
            CURRENT-WORD-NAME I + C@
            WORD-NAME-BUFFER 4 + I + C!
        LOOP
        \ Look up existing function
        CURRENT-MODULE @
        WORD-NAME-BUFFER
        CURRENT-WORD-LEN @ 4 +
        LLVM-MODULE-GET-FUNCTION CURRENT-FUNCTION !
    ELSE
        \ Single word mode: create new function
        102 WORD-NAME-BUFFER 0 + C!  \ 'f'
        117 WORD-NAME-BUFFER 1 + C!  \ 'u'
        110 WORD-NAME-BUFFER 2 + C!  \ 'n'
        99 WORD-NAME-BUFFER 3 + C!  \ 'c'
        CURRENT-MODULE @ CURRENT-CTX @
        WORD-NAME-BUFFER 4
        LLVM-CREATE-FUNCTION CURRENT-FUNCTION !
    THEN

    \ Get function parameters
    CURRENT-FUNCTION @ 0 LLVM-GET-PARAM PARAM-MEMORY !
    CURRENT-FUNCTION @ 1 LLVM-GET-PARAM PARAM-SP !
    CURRENT-FUNCTION @ 2 LLVM-GET-PARAM PARAM-RP !

    \ Create entry block
    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "entry" to WORD-NAME-BUFFER
    101 WORD-NAME-BUFFER 0 + C!  \ 'e'
    110 WORD-NAME-BUFFER 1 + C!  \ 'n'
    116 WORD-NAME-BUFFER 2 + C!  \ 't'
    114 WORD-NAME-BUFFER 3 + C!  \ 'r'
    121 WORD-NAME-BUFFER 4 + C!  \ 'y'
    WORD-NAME-BUFFER 5 LLVM-CREATE-BLOCK CURRENT-BLOCK !

    \ Position at entry
    CURRENT-BUILDER @ CURRENT-BLOCK @ LLVM-POSITION-AT-END

    \ Retrieve AST handle from variable and compile
    CURRENT-AST-HANDLE @ COMPILE-AST-NODE

    \ Add return
    CURRENT-BUILDER @ LLVM-BUILD-RET-VOID

    \ Check if in batch mode
    BATCH-MODE @ IF
        \ Batch mode: don't create JIT yet, return 0 as placeholder
        \ Restore name from return stack and drop
        R> DROP R> DROP
        0  \ Return 0 as placeholder
    ELSE
        \ Single word mode: create JIT and get function pointer
        CURRENT-MODULE @ LLVM-CREATE-JIT

        \ Stack: ( jit-engine-handle )

        \ Get function pointer for "func"
        \ Write "func" to WORD-NAME-BUFFER
        102 WORD-NAME-BUFFER 0 + C!  \ 'f'
        117 WORD-NAME-BUFFER 1 + C!  \ 'u'
        110 WORD-NAME-BUFFER 2 + C!  \ 'n'
        99  WORD-NAME-BUFFER 3 + C!  \ 'c'
        WORD-NAME-BUFFER 4 LLVM-GET-FUNCTION

        \ Stack: ( fn-ptr )

        \ Restore name from return stack and drop
        R> DROP R> DROP
    THEN
    ;

\ =============================================================================
\ TEST
\ =============================================================================

: TEST-COMPILER
    CR ." Compiler ready" CR ;
