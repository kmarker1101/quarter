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
    THEN

    \ For DUP, SWAP, DROP: lowercase + quarter_ prefix
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
        2 PICK I + C@  \ Get char
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
    \ LLVM-BUILD-LOAD expects: ( builder ctx ptr -- value )
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-SP @ LLVM-BUILD-LOAD
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

    \ 4. Create constant 4
    CURRENT-CTX @ 4 LLVM-BUILD-CONST-INT
    \ Stack: ( value sp-val four )

    \ 5. Add: new_sp = sp_val + 4
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
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-SP @ LLVM-BUILD-LOAD

    \ Stack: ( sp-val-handle )

    \ 2. Create constant 4
    CURRENT-CTX @ 4 LLVM-BUILD-CONST-INT

    \ Stack: ( sp-val-handle four-handle )

    \ 3. Subtract: new_sp = sp_val - 4
    CURRENT-BUILDER @ 2 PICK 2 PICK LLVM-BUILD-SUB

    \ Stack: ( sp-val-handle four-handle new-sp-handle )

    \ 4. Store new SP
    CURRENT-BUILDER @ DUP PARAM-SP @ LLVM-BUILD-STORE

    \ Stack: ( sp-val-handle four-handle new-sp-handle )
    NIP NIP \ Drop sp-val and four

    \ Stack: ( new-sp-handle )

    \ 5. GEP to get address: addr = memory + new_sp
    CURRENT-BUILDER @ CURRENT-CTX @ PARAM-MEMORY @ 2 PICK LLVM-BUILD-GEP

    \ Stack: ( new-sp-handle addr-handle )
    NIP \ Drop new-sp-handle

    \ Stack: ( addr-handle )

    \ 6. Load value from address
    CURRENT-BUILDER @ CURRENT-CTX @ ROT LLVM-BUILD-LOAD

    \ Stack: ( value-handle )
    ;

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
    CURRENT-CTX @ 9 PICK LLVM-BUILD-CONST-INT
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
    CURRENT-CTX @ 0 LLVM-BUILD-CONST-INT
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
    CURRENT-CTX @ 0 LLVM-BUILD-CONST-INT ( then else builder pred cond zero )
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
        CURRENT-CTX @ SWAP
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
    CURRENT-MODULE @ CURRENT-CTX @ -ROT
    LLVM-DECLARE-EXTERNAL ;

\ Declare all primitive functions in the module
: DECLARE-ALL-PRIMITIVES
    \ Arithmetic
    COMPILER-SCRATCH 11 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 97 OVER 8+ C! 100 OVER 9+ C! 100 OVER 10+ C! DROP COMPILER-SCRATCH 11 DECLARE-PRIMITIVE  \ quarter_add
    COMPILER-SCRATCH 11 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 115 OVER 8+ C! 117 OVER 9+ C! 98 OVER 10+ C! DROP COMPILER-SCRATCH 11 DECLARE-PRIMITIVE  \ quarter_sub
    COMPILER-SCRATCH 11 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 109 OVER 8+ C! 117 OVER 9+ C! 108 OVER 10+ C! DROP COMPILER-SCRATCH 11 DECLARE-PRIMITIVE  \ quarter_mul
    COMPILER-SCRATCH 11 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 100 OVER 8+ C! 105 OVER 9+ C! 118 OVER 10+ C! DROP COMPILER-SCRATCH 11 DECLARE-PRIMITIVE  \ quarter_div

    \ Stack operations
    COMPILER-SCRATCH 11 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 100 OVER 8+ C! 117 OVER 9+ C! 112 OVER 10+ C! DROP COMPILER-SCRATCH 11 DECLARE-PRIMITIVE  \ quarter_dup
    COMPILER-SCRATCH 12 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 115 OVER 8+ C! 119 OVER 9+ C! 97 OVER 10+ C! 112 OVER 11+ C! DROP COMPILER-SCRATCH 12 DECLARE-PRIMITIVE  \ quarter_swap
    COMPILER-SCRATCH 12 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 100 OVER 8+ C! 114 OVER 9+ C! 111 OVER 10+ C! 112 OVER 11+ C! DROP COMPILER-SCRATCH 12 DECLARE-PRIMITIVE  \ quarter_drop

    \ Output
    COMPILER-SCRATCH 11 111 OVER C! 117 OVER 1+ C! 97 OVER 2+ C! 114 OVER 3+ C! 116 OVER 4+ C! 101 OVER 5+ C! 114 OVER 6+ C! 95 OVER 7+ C! 100 OVER 8+ C! 111 OVER 9+ C! 116 OVER 10+ C! DROP COMPILER-SCRATCH 11 DECLARE-PRIMITIVE  \ quarter_dot
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
