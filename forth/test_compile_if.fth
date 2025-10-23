\ Test compiling IF/THEN/ELSE in Forth compiler

\ First test: Simple number - should already work
\ : TEST-NUM 42 ;

\ Second test: IF/THEN (no ELSE)
\ : TEST-IF 5 3 < IF 99 THEN ;

\ Third test: IF/THEN/ELSE
\ : TEST-IF-ELSE 1 IF 42 ELSE 99 THEN ;

CR ." Compiler test loaded" CR
