\ REPL - Read-Eval-Print Loop implemented in Forth
\ This provides a self-hosting REPL using READLINE primitive

\ Constants for buffer sizes
1024 CONSTANT INPUT-BUFFER-SIZE
256 CONSTANT PROMPT-SIZE

\ Allocate buffers
VARIABLE INPUT-BUFFER INPUT-BUFFER-SIZE ALLOT
VARIABLE PROMPT-BUFFER PROMPT-SIZE ALLOT

\ Format the prompt string
\ Returns: ( -- addr len )
: FORMAT-PROMPT
  S" quarter> " PROMPT-BUFFER SWAP
  DUP >R  \ Save length
  PROMPT-BUFFER SWAP CMOVE
  PROMPT-BUFFER R>
;

\ Main REPL loop
\ Continuously reads and executes input until EOF/interrupt
: REPL
  BEGIN
    FORMAT-PROMPT READLINE  \ ( -- line-addr line-len flag )
  WHILE                     \ Continue while flag is true
    \ We have input: line-addr line-len on stack
    2DUP HISTORY-ADD        \ Add to history

    \ Execute the input using EVALUATE
    EVALUATE
    ."  ok" CR
  REPEAT
  2DROP                     \ Clean up addr/len from failed READLINE
;

\ Initialize and start the REPL
: QUARTER-REPL
  \ Load history from home directory
  S" .quarter_history" HISTORY-LOAD DROP

  \ Run the main REPL loop
  REPL

  \ Save history on exit
  S" .quarter_history" HISTORY-SAVE DROP

  \ Print goodbye message
  ." Goodbye!" CR
;
