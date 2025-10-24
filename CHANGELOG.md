# Changelog

All notable changes to Quarter Forth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-24

### Major Changes

#### Self-Hosting Compiler
- **Removed duplicate Rust LLVM compiler** (#52, #53)
  - Deleted 2,307 lines of duplicate code
  - Now uses only the Forth compiler (766 lines)
  - Achieved true self-hosting: compiler written in target language
  - Single compilation path for all user-defined words

#### 64-bit Migration
- **Migrated from 32-bit to 64-bit architecture** (#51)
  - All cells now 8 bytes (i64)
  - Memory increased to 8MB (matching gforth)
  - Stack pointers use 64-bit addressing
  - Better performance and compatibility

### Added

#### Optimization Features
- **Tail call optimization** (#50)
  - Optimizes recursive calls in tail position
  - Prevents stack overflow in deep recursion
- **EXIT word** - Early return from word definitions (#22)
- **LEAVE word** - Early exit from loops

#### LLVM JIT Compilation
- **Self-hosting Forth compiler** (#29, #44, #45)
  - 766-line compiler written entirely in Forth
  - Compiles AST to LLVM IR to native code
  - Uses LLVM-* primitives for code generation
- **Control flow compilation** (#43)
  - IF/THEN/ELSE compilation
  - Loop compilation (BEGIN/UNTIL, DO/LOOP)
  - Recursive word calls
- **Call optimization** (#42)
  - JIT-compiled words can call each other
  - Primitive calls optimized
- **LLVM infrastructure** (#26, #41)
  - 30 LLVM primitives (LLVM-CREATE-*, LLVM-BUILD-*)
  - 10 AST inspection primitives
  - Handle-based API for Forth

#### Standard Library
- **Moved primitives to Forth stdlib** (#11, #40)
  - 40 words now defined in Forth
  - Minimal Rust primitives (43 words)
  - Auto-loading stdlib on startup (#9, #39)

#### Memory & Variables
- **Stacks in memory** (#24, #31)
  - Data stack at 0x000000-0x00FFFF (64KB)
  - Return stack at 0x010000-0x01FFFF (64KB)
  - User memory at 0x020000-0x7FFFFF (~7.5MB)
- **Variables and constants** (#7, #35)
  - VARIABLE, CONSTANT, CREATE
  - HERE, ALLOT, , (comma)
- **Memory access words** (#4, #25)
  - !, @, C!, C@ for direct memory access
  - Stack pointer access (SP@, SP!, RP@, RP!)

#### I/O & Strings
- **String literals** (#37)
  - S" for creating strings on stack
  - INCLUDED for loading files from string
- **Multi-line REPL** (#37)
  - Compilation mode for multi-line definitions
  - forth-mode compatible

#### Stack Operations
- **Double-stack operations** (#6, #34)
  - 2DUP, 2DROP, 2SWAP, 2OVER

### Improved

#### Code Quality
- **Fixed 25 clippy warnings** (#54, #55)
  - Reduced warnings from 29 to 4 (86% reduction)
  - Added Default implementations for core types
  - Cleaner error handling with ? operator
  - Better Rust best practices

#### Testing
- **Forth testing framework** (#47)
  - Fixed test framework for stdlib testing
  - 116 tests passing

#### Comments
- **Standard Forth comment behavior** (#16, #36)
  - Backslash comments (\)
  - Parenthesis comments ( )

### Fixed
- **Critical compiler bugs** (#45, #46)
  - Fixed LLVM-BUILD-CALL argument order
  - Fixed Forth compiler edge cases
- **Cleanup** - General code cleanup and improvements

### Performance
- **Benchmarking suite** (#27)
  - Added --no-jit flag for comparisons
  - Benchmark tests for JIT vs interpreted

## [0.1.0] - 2025-10-23

### Initial Release
- Basic Forth interpreter
- Core primitives (arithmetic, stack, logic, I/O)
- User-defined words (: and ;)
- Control flow (IF/THEN/ELSE, BEGIN/UNTIL, DO/LOOP)
- Return stack operations (>R, R>, R@)
- Bitwise operations
- File loading (INCLUDE)
- REPL with rustyline

## [0.0.0] - 2025-10-20

### Project Start
- Initial commit
- Basic project structure

[0.2.0]: https://github.com/kmarker1101/quarter/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kmarker1101/quarter/compare/v0.0.0...v0.1.0
[0.0.0]: https://github.com/kmarker1101/quarter/releases/tag/v0.0.0
