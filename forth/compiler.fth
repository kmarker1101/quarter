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

\ =============================================================================
\ AST COMPILATION
\ =============================================================================

\ Forward declaration for recursion
: COMPILE-AST-NODE ;

\ Compile DO/LOOP (type 7)
\ Stack: ( ast-handle -- )
: COMPILE-DO-LOOP
    \ Get loop body and increment
    DUP AST-LOOP-BODY SWAP AST-LOOP-INCREMENT
    \ Stack: ( body-handle increment )

    \ Pop start and limit from runtime stack
    COMPILE-POP COMPILE-POP  \ start, limit
    \ Stack: ( body increment start-handle limit-handle )

    \ Get pre-loop block
    CURRENT-BUILDER @ LLVM-GET-INSERT-BLOCK
    \ Stack: ( body increment start limit preloop-block )

    \ Create loop block
    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "do_loop" to WORD-NAME-BUFFER
    100 WORD-NAME-BUFFER 0 + C!  \ 'd'
    111 WORD-NAME-BUFFER 1 + C!  \ 'o'
    95  WORD-NAME-BUFFER 2 + C!  \ '_'
    108 WORD-NAME-BUFFER 3 + C!  \ 'l'
    111 WORD-NAME-BUFFER 4 + C!  \ 'o'
    111 WORD-NAME-BUFFER 5 + C!  \ 'o'
    112 WORD-NAME-BUFFER 6 + C!  \ 'p'
    WORD-NAME-BUFFER 7 LLVM-CREATE-BLOCK

    \ Create exit block
    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "do_exit" to WORD-NAME-BUFFER
    100 WORD-NAME-BUFFER 0 + C!  \ 'd'
    111 WORD-NAME-BUFFER 1 + C!  \ 'o'
    95  WORD-NAME-BUFFER 2 + C!  \ '_'
    101 WORD-NAME-BUFFER 3 + C!  \ 'e'
    120 WORD-NAME-BUFFER 4 + C!  \ 'x'
    105 WORD-NAME-BUFFER 5 + C!  \ 'i'
    116 WORD-NAME-BUFFER 6 + C!  \ 't'
    WORD-NAME-BUFFER 7 LLVM-CREATE-BLOCK
    \ Stack: ( body incr start limit preloop loop-block exit-block )

    \ Jump to loop
    2 PICK CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

    \ Position at loop
    OVER CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END

    \ Create PHI for loop index
    CURRENT-BUILDER @ CURRENT-CTX @
    \ Write "i" to WORD-NAME-BUFFER
    105 WORD-NAME-BUFFER C!
    WORD-NAME-BUFFER 1 LLVM-BUILD-PHI
    \ Stack: ( body incr start limit preloop loop exit phi )

    \ Add incoming from preloop
    DUP 7 PICK 6 PICK LLVM-PHI-ADD-INCOMING

    \ Compile body
    7 PICK COMPILE-AST-NODE

    \ Get loop-end block
    CURRENT-BUILDER @ LLVM-GET-INSERT-BLOCK
    \ Stack: ( body incr start limit preloop loop exit phi loop-end )

    \ Increment: next = phi + increment
    CURRENT-BUILDER @ 2 PICK
    CURRENT-CTX @ 9 PICK 32 LLVM-BUILD-CONST-INT
    LLVM-BUILD-ADD
    \ Stack: ( body incr start limit preloop loop exit phi loop-end next )

    \ Compare: next < limit (SLT=2)
    CURRENT-BUILDER @ 2 2 PICK 8 PICK LLVM-BUILD-ICMP
    \ Stack: ( body incr start limit preloop loop exit phi loop-end next cond )

    \ Add PHI incoming from loop-end
    4 PICK 3 PICK 4 PICK LLVM-PHI-ADD-INCOMING

    \ Conditional branch
    CURRENT-BUILDER @ SWAP 7 PICK 4 PICK LLVM-BUILD-COND-BR

    \ Position at exit
    3 PICK CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END

    \ Clean up
    DROP DROP DROP DROP DROP DROP DROP DROP DROP DROP ;

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
    \ Get then and else branches
    DUP AST-IF-THEN ( ast then-handle )
    SWAP AST-IF-ELSE ( then-handle else-handle-or-0 )

    \ Pop condition from stack (generates LLVM IR)
    COMPILE-POP ( then-handle else-handle cond-value-handle )

    \ Compare to zero (Forth: 0=false, nonzero=true)
    \ LLVM-BUILD-ICMP needs: builder predicate lhs rhs -> result
    \ Predicate 1 = NE (not equal)
    CURRENT-BUILDER @ 1  ( then else cond builder pred )
    ROT ( then else builder pred cond )
    CURRENT-CTX @ 0 32 LLVM-BUILD-CONST-INT ( then else builder pred cond zero )
    LLVM-BUILD-ICMP ( then else bool-handle )

    \ Create basic blocks
    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "then" to WORD-NAME-BUFFER
    116 WORD-NAME-BUFFER 0 + C!  \ 't'
    104 WORD-NAME-BUFFER 1 + C!  \ 'h'
    101 WORD-NAME-BUFFER 2 + C!  \ 'e'
    110 WORD-NAME-BUFFER 3 + C!  \ 'n'
    WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK ( then else bool then-block )

    CURRENT-CTX @ CURRENT-FUNCTION @
    \ Write "merge" to WORD-NAME-BUFFER
    109 WORD-NAME-BUFFER 0 + C!  \ 'm'
    101 WORD-NAME-BUFFER 1 + C!  \ 'e'
    114 WORD-NAME-BUFFER 2 + C!  \ 'r'
    103 WORD-NAME-BUFFER 3 + C!  \ 'g'
    101 WORD-NAME-BUFFER 4 + C!  \ 'e'
    WORD-NAME-BUFFER 5 LLVM-CREATE-BLOCK ( then else bool then-block merge-block )

    \ Check if we have else branch
    OVER 0 = IF
        \ No ELSE: build conditional branch to then or merge
        2SWAP DROP ( bool then-block merge-block )
        2 PICK CURRENT-BUILDER @ SWAP ( bool then merge bool builder )
        -ROT ( bool builder bool then merge )
        LLVM-BUILD-COND-BR

        \ Compile THEN branch
        OVER CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
        SWAP COMPILE-AST-NODE

        \ Branch to merge
        DUP CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

        \ Position at merge
        CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
    ELSE
        \ Have ELSE: create else block too
        CURRENT-CTX @ CURRENT-FUNCTION @
        \ Write "else" to WORD-NAME-BUFFER
        101 WORD-NAME-BUFFER 0 + C!  \ 'e'
        108 WORD-NAME-BUFFER 1 + C!  \ 'l'
        115 WORD-NAME-BUFFER 2 + C!  \ 's'
        101 WORD-NAME-BUFFER 3 + C!  \ 'e'
        WORD-NAME-BUFFER 4 LLVM-CREATE-BLOCK ( then else bool then-block merge-block else-block )

        \ Build conditional branch (if bool then then-block else else-block)
        5 PICK CURRENT-BUILDER @ ( then else bool then merge else bool builder )
        SWAP 4 PICK ( then else bool then merge else builder bool then )
        SWAP OVER ( then else bool then merge else builder then bool )
        PICK ( then else bool then merge else builder bool then else )
        LLVM-BUILD-COND-BR

        \ Compile THEN branch
        2 PICK ( then else bool then merge else then )
        CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
        5 PICK COMPILE-AST-NODE

        \ Branch to merge
        2 PICK CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

        \ Compile ELSE branch
        DUP CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
        3 PICK COMPILE-AST-NODE

        \ Branch to merge
        2 PICK CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

        \ Position at merge
        NIP NIP NIP ( then merge )
        CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END
        DROP
    THEN ;

\ Main compiler - handles all AST node types recursively
\ Redefine the forward-declared COMPILE-AST-NODE
: COMPILE-AST-NODE ( ast-handle -- )
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

        \ Not an inlinable primitive - do normal function call
        \ Map to primitive name
        WORD-NAME-BUFFER SWAP MAP-WORD-NAME
        \ Stack: ( prim-addr prim-len )

        \ Look up function in module
        CURRENT-MODULE @ -ROT
        LLVM-MODULE-GET-FUNCTION

        \ Stack: ( fn-handle )

        \ Call with (memory, sp, rp) parameters
        CURRENT-BUILDER @ SWAP
        PARAM-MEMORY @ PARAM-SP @ PARAM-RP @ 3
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
;

\ =============================================================================
\ COMPILER ENTRY POINT
\ =============================================================================

\ Compile a word from its AST
\ Returns JIT function pointer
: COMPILE-WORD ( ast-handle name-addr name-len -- fn-ptr )
    \ Save name for later (we'll need it for JIT lookup)
    >R >R  \ Save name-addr and name-len on return stack

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

    \ Create function
    CURRENT-MODULE @ CURRENT-CTX @
    \ Write "func" to WORD-NAME-BUFFER
    102 WORD-NAME-BUFFER 0 + C!  \ 'f'
    117 WORD-NAME-BUFFER 1 + C!  \ 'u'
    110 WORD-NAME-BUFFER 2 + C!  \ 'n'
    99 WORD-NAME-BUFFER 3 + C!  \ 'c'
    WORD-NAME-BUFFER 4 LLVM-CREATE-FUNCTION CURRENT-FUNCTION !

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

    \ Compile the AST
    COMPILE-AST-NODE

    \ Add return
    CURRENT-BUILDER @ LLVM-BUILD-RET-VOID

    \ Create JIT execution engine
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

    \ Restore name from return stack and drop (we don't need it anymore)
    R> DROP R> DROP

    \ Return function pointer
    ;

\ =============================================================================
\ TEST
\ =============================================================================

: TEST-COMPILER
    CR ." Compiler ready" CR ;
