# Testing the Forth REPL

This document provides manual testing instructions for the self-hosting Forth REPL implementation (Issue #71).

## Prerequisites

Build the project:
```bash
cargo build
```

## Starting the REPL

```bash
./target/debug/quarter
```

You should see:
```
Forth Interpreter v0.2
Type CTRL-C or CTRL-D to exit
quarter>
```

## Test Cases

### 1. Basic Arithmetic

```forth
quarter> 5 3 + .
8 ok
quarter> 10 2 * .
20 ok
```

### 2. Stack Operations

```forth
quarter> 1 2 3 .S
<3> 1 2 3 ok
quarter> DROP .S
<2> 1 2 ok
quarter> SWAP .S
<2> 2 1 ok
```

### 3. Single-Line Word Definitions

```forth
quarter> : SQUARE DUP * ;
ok
quarter> 5 SQUARE .
25 ok
```

### 4. Multi-Line Word Definitions

```forth
quarter> : CUBE
compiled DUP
compiled SQUARE
compiled *
compiled ;
ok
quarter> 3 CUBE .
27 ok
```

**Expected behavior:**
- Prompt changes to `compiled` when in multi-line mode
- Definition completes when `;` is entered
- Word is immediately available for use

### 5. EVALUATE Primitive

```forth
quarter> : TEST-EVAL S" 2 2 + ." EVALUATE ;
ok
quarter> TEST-EVAL
4 ok
```

### 6. Control Flow in Definitions

```forth
quarter> : ABS
compiled DUP 0<
compiled IF NEGATE THEN
compiled ;
ok
quarter> -5 ABS .
5 ok
quarter> 5 ABS .
5 ok
```

### 7. Loops in Definitions

```forth
quarter> : COUNTDOWN
compiled BEGIN
compiled DUP .
compiled 1 -
compiled DUP 0=
compiled UNTIL
compiled DROP
compiled ;
ok
quarter> 5 COUNTDOWN
5 4 3 2 1 ok
```

### 8. Command History

The REPL should save command history to `.quarter_history` in your home directory.

Test:
1. Enter some commands
2. Exit the REPL (Ctrl-D or Ctrl-C)
3. Restart the REPL
4. Press Up arrow - you should see previous commands

### 9. Error Handling

Test various error conditions:

```forth
quarter> UNDEFINED-WORD
Unknown word: UNDEFINED-WORD
quarter> : BAD-DEF FOO BAR ;
Undefined word: FOO
quarter>
```

**Note:** Error handling is basic - errors are printed but don't crash the REPL.

### 10. Complex Multi-Line Definition

```forth
quarter> : FACTORIAL
compiled DUP 0= IF
compiled DROP 1
compiled ELSE
compiled DUP 1 - FACTORIAL *
compiled THEN
compiled ;
ok
quarter> 5 FACTORIAL .
120 ok
```

## Known Limitations

1. **EVALUATE Context**: EVALUATE only works when the REPL is running. It cannot be used in files loaded from command line (e.g., `quarter myfile.fth`).

2. **Error Recovery**: Errors in multi-line definitions may leave the REPL in compilation mode. If this happens, type `;` on a new line to exit compilation mode.

3. **No CATCH/THROW**: Advanced error handling with CATCH and THROW is not yet implemented.

4. **Limited History**: History is saved/loaded but editing features depend on rustyline's capabilities.

## Troubleshooting

### REPL won't start
- Check that `stdlib/repl.fth` exists
- Try `cargo clean && cargo build`

### Compilation mode stuck
- Type `;` on a new line to force exit from compilation mode
- Restart the REPL if necessary

### Commands not working
- Ensure you're using the latest build: `cargo build`
- Check for typos (Forth is case-insensitive but punctuation matters)

### History not saving
- Check permissions on home directory
- Look for `.quarter_history` file in home directory

## Success Criteria

The Forth REPL is working correctly if:
- ✅ REPL starts and shows prompt
- ✅ Basic arithmetic works
- ✅ Single-line word definitions work
- ✅ Multi-line definitions work with prompt change
- ✅ EVALUATE works within defined words
- ✅ Control flow and loops work in definitions
- ✅ Command history persists across sessions
- ✅ Errors are handled gracefully (don't crash REPL)

## Reporting Issues

If you encounter issues:
1. Note the exact commands entered
2. Note any error messages
3. Check if `cargo test` passes
4. Report on issue #71 with details
