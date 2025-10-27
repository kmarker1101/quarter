\ REPL - Read-Eval-Print Loop implemented in Forth
\ This provides a self-hosting REPL using READLINE primitive

\ Constants for buffer sizes
8192 CONSTANT COMPILE-BUFFER-SIZE

\ Allocate compilation buffer
VARIABLE COMPILE-BUFFER COMPILE-BUFFER-SIZE ALLOT

\ Compilation mode state
VARIABLE COMPILING         \ Flag: 0 = normal, -1 = compiling
VARIABLE COMPILE-POS       \ Current position in compile buffer

\ Initialize compilation mode
: INIT-COMPILE-MODE
  0 COMPILING !
  0 COMPILE-POS !
;

\ Check if line starts with `:` (entering compilation mode)
\ ( addr len -- flag )
: STARTS-WITH-COLON
  DUP 0> IF
    SWAP C@ 58 =  \ Check if first char is ':' (ASCII 58)
    SWAP DROP  \ Drop the length, keep only the flag
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
  \ Save length for later use
  DUP >R

  \ Calculate destination: COMPILE-BUFFER + COMPILE-POS
  COMPILE-BUFFER COMPILE-POS @ +

  \ Rearrange for CMOVE: src dest count
  SWAP

  \ Copy line to buffer
  CMOVE

  \ Add space after the line
  COMPILE-BUFFER COMPILE-POS @ R@ + +
  32 SWAP C!

  \ Update compile position: pos + len + 1 (for space)
  R> 1 + COMPILE-POS @ + COMPILE-POS !
;

\ Main REPL with multi-line support
: SIMPLE-REPL
  BEGIN
    COMPILING @ IF
      S" compiled " READLINE
    ELSE
      S" quarter> " READLINE
    THEN
  WHILE
    2DUP HISTORY-ADD
    COMPILING @ IF
      \ In compilation mode - accumulate lines
      2DUP APPEND-TO-COMPILE-BUFFER
      2DUP CONTAINS-SEMICOLON IF
        \ End compilation - evaluate buffer
        COMPILE-BUFFER COMPILE-POS @ EVALUATE
        ."  ok" CR
        INIT-COMPILE-MODE
      THEN
      2DROP
    ELSE
      \ Normal mode - check if entering compilation
      2DUP STARTS-WITH-COLON IF
        2DUP CONTAINS-SEMICOLON IF
          \ Single-line definition
          EVALUATE
          ."  ok" CR
        ELSE
          \ Multi-line definition - enter compilation mode
          -1 COMPILING !
          APPEND-TO-COMPILE-BUFFER
        THEN
      ELSE
        \ Normal execution
        EVALUATE
        ."  ok" CR
      THEN
    THEN
  REPEAT
  2DROP  \ Clean up failed READLINE
;

\ Initialize and start the REPL
: QUARTER-REPL
  \ Load history from home directory
  S" .quarter_history" HISTORY-LOAD DROP

  \ Initialize compilation mode
  INIT-COMPILE-MODE

  \ Run enhanced REPL with multi-line support
  SIMPLE-REPL

  \ Save history on exit
  S" .quarter_history" HISTORY-SAVE DROP

  \ Print goodbye message
  CR ." Goodbye!" CR
;
