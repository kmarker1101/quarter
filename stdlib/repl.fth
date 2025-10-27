\ REPL - Read-Eval-Print Loop implemented in Forth
\ This provides a self-hosting REPL using READLINE primitive

\ Constants for buffer sizes
4096 CONSTANT INPUT-BUFFER-SIZE
256 CONSTANT PROMPT-SIZE
8192 CONSTANT COMPILE-BUFFER-SIZE

\ Allocate buffers
VARIABLE INPUT-BUFFER INPUT-BUFFER-SIZE ALLOT
VARIABLE PROMPT-BUFFER PROMPT-SIZE ALLOT
VARIABLE COMPILE-BUFFER COMPILE-BUFFER-SIZE ALLOT

\ Compilation mode state
VARIABLE COMPILING         \ Flag: 0 = normal, -1 = compiling
VARIABLE COMPILE-POS       \ Current position in compile buffer

\ Initialize compilation mode
: INIT-COMPILE-MODE
  0 COMPILING !
  0 COMPILE-POS !
;

\ Check if a line contains a word (case-insensitive)
\ ( addr len word-addr word-len -- flag )
: LINE-CONTAINS-WORD
  >R >R  \ Save word address and length
  BEGIN
    DUP 0>  \ While line length > 0
  WHILE
    \ Check if current position matches word
    2DUP R@ R> 2SWAP  \ Prepare for comparison
    >R >R  \ Save word addr/len again
    \ TODO: Implement case-insensitive comparison
    \ For now, just scan for space-separated tokens
    DROP DROP  \ Simplified - just drop for now
    1 -  \ Decrease line length
    SWAP 1 + SWAP  \ Advance line pointer
  REPEAT
  2DROP R> R> 2DROP
  0  \ Return false for now (TODO: implement properly)
;

\ Check if line starts with `:` (entering compilation mode)
\ ( addr len -- flag )
: STARTS-WITH-COLON
  DUP 0> IF
    SWAP C@ 58 =  \ Check if first char is ':' (ASCII 58)
  ELSE
    DROP 0
  THEN
;

\ Check if line contains `;` (ending compilation mode)
\ ( addr len -- flag )
: CONTAINS-SEMICOLON
  0 >R  \ Flag on return stack
  BEGIN
    DUP 0>
  WHILE
    OVER C@ 59 = IF  \ Check for ';' (ASCII 59)
      R> DROP -1 >R  \ Set flag to true
    THEN
    1 - SWAP 1 + SWAP
  REPEAT
  2DROP R>
;

\ Append line to compile buffer with space separator
\ ( line-addr line-len -- )
: APPEND-TO-COMPILE-BUFFER
  COMPILE-POS @ >R  \ Save current position

  \ Copy line to compile buffer
  COMPILE-BUFFER R@ +  \ Destination
  SWAP  \ Get length on top
  DUP >R  \ Save length
  CMOVE

  \ Add space after the line
  COMPILE-BUFFER R> R@ + +  \ Position after copied text
  32 SWAP C!  \ Store space

  \ Update compile position
  R> 1 + COMPILE-POS @ + COMPILE-POS !
;

\ Format the prompt string based on mode
\ Returns: ( -- addr len )
: FORMAT-PROMPT
  COMPILING @ IF
    S" compiled " PROMPT-BUFFER SWAP
  ELSE
    S" quarter> " PROMPT-BUFFER SWAP
  THEN
  DUP >R  \ Save length
  PROMPT-BUFFER SWAP CMOVE
  PROMPT-BUFFER R>
;

\ Main REPL loop
\ Continuously reads and executes input until EOF/interrupt
: REPL
  INIT-COMPILE-MODE
  BEGIN
    FORMAT-PROMPT READLINE  \ ( -- line-addr line-len flag )
  WHILE                     \ Continue while flag is true
    \ We have input: line-addr line-len on stack
    2DUP HISTORY-ADD        \ Add to history

    COMPILING @ IF
      \ In compilation mode - accumulate line
      2DUP APPEND-TO-COMPILE-BUFFER

      \ Check if line contains ;
      CONTAINS-SEMICOLON IF
        \ End compilation - evaluate accumulated buffer
        COMPILE-BUFFER COMPILE-POS @ EVALUATE
        ."  ok" CR
        INIT-COMPILE-MODE
      THEN
      2DROP
    ELSE
      \ Normal mode - check if entering compilation
      2DUP STARTS-WITH-COLON IF
        \ Check if also contains ; (single-line definition)
        2DUP CONTAINS-SEMICOLON IF
          \ Single-line definition - just evaluate
          EVALUATE
          ."  ok" CR
        ELSE
          \ Multi-line definition - enter compilation mode
          -1 COMPILING !
          APPEND-TO-COMPILE-BUFFER
        THEN
        2DROP
      ELSE
        \ Normal execution
        EVALUATE
        ."  ok" CR
      THEN
    THEN
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
