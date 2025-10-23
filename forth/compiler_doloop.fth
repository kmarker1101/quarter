\ DO/LOOP Compilation (temporarily in separate file for clarity)
\ This will be integrated into compiler.fth

\ Compile DO/LOOP (type 7)
\ Stack: ( ast-handle -- )
: COMPILE-DO-LOOP
    \ Get loop body and increment
    DUP AST-LOOP-BODY SWAP
    AST-LOOP-INCREMENT

    \ Stack: ( body-handle increment )

    \ Pop start and limit from runtime stack
    COMPILE-POP  \ start
    COMPILE-POP  \ limit

    \ Stack: ( body increment start-handle limit-handle )

    \ Get pre-loop block (where start/limit were computed)
    CURRENT-BUILDER @ LLVM-GET-INSERT-BLOCK

    \ Stack: ( body increment start limit preloop-block )

    \ Create loop and exit blocks
    CURRENT-CTX @ CURRENT-FUNCTION @
    WORD-NAME-BUFFER DUP 7
    100 OVER C! 1+  \ 'd'
    111 OVER C! 1+  \ 'o'
     95 OVER C! 1+  \ '_'
    108 OVER C! 1+  \ 'l'
    111 OVER C! 1+  \ 'o'
    111 OVER C! 1+  \ 'o'
    112 OVER C! DROP  \ 'p'
    7 LLVM-CREATE-BLOCK

    CURRENT-CTX @ CURRENT-FUNCTION @
    WORD-NAME-BUFFER DUP 7
    100 OVER C! 1+  \ 'd'
    111 OVER C! 1+  \ 'o'
     95 OVER C! 1+  \ '_'
    101 OVER C! 1+  \ 'e'
    120 OVER C! 1+  \ 'x'
    105 OVER C! 1+  \ 'i'
    116 OVER C! DROP  \ 't'
    7 LLVM-CREATE-BLOCK

    \ Stack: ( body incr start limit preloop loop-block exit-block )

    \ Jump to loop
    2 PICK CURRENT-BUILDER @ SWAP LLVM-BUILD-BR

    \ Position at loop block
    OVER CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END

    \ Create PHI node for loop index
    CURRENT-BUILDER @ CURRENT-CTX @
    WORD-NAME-BUFFER DUP 1
    105 OVER C! DROP \ 'i'
    1 LLVM-BUILD-PHI

    \ Stack: ( body incr start limit preloop loop exit phi-handle )

    \ Add incoming from preloop: phi.add_incoming(start, preloop)
    DUP 7 PICK 6 PICK LLVM-PHI-ADD-INCOMING

    \ Stack: ( body incr start limit preloop loop exit phi )

    \ Compile loop body
    \ TODO: Need to make loop index accessible for I word
    \ For now, just compile the body
    7 PICK COMPILE-AST-NODE

    \ Get loop-end block (after compiling body)
    CURRENT-BUILDER @ LLVM-GET-INSERT-BLOCK

    \ Stack: ( body incr start limit preloop loop exit phi loop-end )

    \ Increment loop index: next_index = phi + increment
    CURRENT-BUILDER @ 2 PICK
    CURRENT-CTX @ 9 PICK LLVM-BUILD-CONST-INT
    LLVM-BUILD-ADD

    \ Stack: ( body incr start limit preloop loop exit phi loop-end next-index )

    \ Compare: next_index < limit (predicate 2 = SLT)
    CURRENT-BUILDER @ 2
    2 PICK 8 PICK
    LLVM-BUILD-ICMP

    \ Stack: ( body incr start limit preloop loop exit phi loop-end next cond )

    \ Add PHI incoming from loop-end: phi.add_incoming(next_index, loop-end)
    4 PICK 3 PICK 4 PICK LLVM-PHI-ADD-INCOMING

    \ Stack: ( body incr start limit preloop loop exit phi loop-end next cond )

    \ Conditional branch: if cond then loop else exit
    CURRENT-BUILDER @ SWAP
    7 PICK 4 PICK
    LLVM-BUILD-COND-BR

    \ Position at exit
    3 PICK CURRENT-BUILDER @ SWAP LLVM-POSITION-AT-END

    \ Clean up stack
    DROP DROP DROP DROP
    DROP DROP DROP DROP
    DROP DROP ;
