/**
 * Quarter Forth Runtime Library
 *
 * Provides runtime support for compiled Forth programs.
 * This library must be linked with compiled Forth object files
 * to create standalone executables.
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

// Memory layout constants (must match Quarter's memory layout)
#define MEMORY_SIZE (8 * 1024 * 1024)  // 8MB
#define STACK_BASE 0x000000
#define STACK_SIZE 0x010000  // 64KB
#define RSTACK_BASE 0x010000
#define RSTACK_SIZE 0x010000  // 64KB

// Global runtime state
static uint8_t* g_memory = NULL;
static size_t g_sp = STACK_BASE;      // Data stack pointer
static size_t g_rp = RSTACK_BASE;     // Return stack pointer

/**
 * Initialize the runtime environment
 * Allocates memory and initializes stack pointers
 */
void quarter_runtime_init(void) {
    g_memory = (uint8_t*)calloc(MEMORY_SIZE, 1);
    if (!g_memory) {
        fprintf(stderr, "Failed to allocate memory\n");
        exit(1);
    }
    g_sp = STACK_BASE;
    g_rp = RSTACK_BASE;
}

/**
 * Cleanup the runtime environment
 */
void quarter_runtime_cleanup(void) {
    if (g_memory) {
        free(g_memory);
        g_memory = NULL;
    }
}

/**
 * Get pointers to runtime state for Forth code
 */
void quarter_runtime_get_state(uint8_t** memory, size_t** sp, size_t** rp) {
    *memory = g_memory;
    *sp = &g_sp;
    *rp = &g_rp;
}

// ============================================================================
// Stack Helper Functions
// ============================================================================

static inline void push_cell(uint8_t* memory, size_t* sp, int64_t value) {
    *(int64_t*)(memory + *sp) = value;
    *sp += 8;
}

static inline int64_t pop_cell(uint8_t* memory, size_t* sp) {
    if (*sp <= STACK_BASE) {
        fprintf(stderr, "Stack underflow\n");
        exit(1);
    }
    *sp -= 8;
    return *(int64_t*)(memory + *sp);
}

static inline int64_t peek_cell(uint8_t* memory, size_t* sp, size_t offset) {
    if (*sp <= STACK_BASE + offset * 8) {
        fprintf(stderr, "Stack underflow\n");
        exit(1);
    }
    return *(int64_t*)(memory + *sp - 8 - offset * 8);
}

// ============================================================================
// I/O Primitives (called by compiled Forth code)
// ============================================================================

/**
 * . (DOT) - Print signed integer
 * Stack: ( n -- )
 */
void quarter_dot(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;  // Unused
    int64_t value = pop_cell(memory, sp);
    printf("%lld ", (long long)value);
}

/**
 * U. - Print unsigned integer
 * Stack: ( u -- )
 */
void quarter_u_dot(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    uint64_t value = (uint64_t)pop_cell(memory, sp);
    printf("%llu ", (unsigned long long)value);
}

/**
 * EMIT - Output character
 * Stack: ( c -- )
 */
void quarter_emit(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t value = pop_cell(memory, sp);
    putchar((int)value);
}

/**
 * KEY - Read character
 * Stack: ( -- c )
 */
void quarter_key(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int c = getchar();
    push_cell(memory, sp, (int64_t)c);
}

/**
 * CR - Print newline
 * Stack: ( -- )
 */
void quarter_cr(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)memory;
    (void)sp;
    (void)rp;
    putchar('\n');
}

/**
 * SPACE - Print space
 * Stack: ( -- )
 */
void quarter_space(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)memory;
    (void)sp;
    (void)rp;
    putchar(' ');
}

// ============================================================================
// Stack Primitives
// ============================================================================

/**
 * DUP - Duplicate top of stack
 * Stack: ( n -- n n )
 */
void quarter_dup(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t value = peek_cell(memory, sp, 0);
    push_cell(memory, sp, value);
}

/**
 * DROP - Remove top of stack
 * Stack: ( n -- )
 */
void quarter_drop(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    pop_cell(memory, sp);
}

/**
 * SWAP - Swap top two stack items
 * Stack: ( a b -- b a )
 */
void quarter_swap(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, b);
    push_cell(memory, sp, a);
}

/**
 * OVER - Copy second stack item to top
 * Stack: ( a b -- a b a )
 */
void quarter_over(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t a = peek_cell(memory, sp, 1);
    push_cell(memory, sp, a);
}

/**
 * ROT - Rotate top three items
 * Stack: ( a b c -- b c a )
 */
void quarter_rot(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t c = pop_cell(memory, sp);
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, b);
    push_cell(memory, sp, c);
    push_cell(memory, sp, a);
}

// ============================================================================
// Arithmetic Primitives
// ============================================================================

/**
 * + - Addition
 * Stack: ( a b -- a+b )
 */
void quarter_add(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, a + b);
}

/**
 * - - Subtraction
 * Stack: ( a b -- a-b )
 */
void quarter_sub(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, a - b);
}

/**
 * * - Multiplication
 * Stack: ( a b -- a*b )
 */
void quarter_mul(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, a * b);
}

/**
 * / - Division
 * Stack: ( a b -- a/b )
 */
void quarter_div(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    if (b == 0) {
        fprintf(stderr, "Division by zero\n");
        exit(1);
    }
    push_cell(memory, sp, a / b);
}

/**
 * NEGATE - Negate top of stack
 * Stack: ( n -- -n )
 */
void quarter_negate(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t value = pop_cell(memory, sp);
    push_cell(memory, sp, -value);
}

// ============================================================================
// Comparison Primitives
// ============================================================================

/**
 * < - Less than
 * Stack: ( a b -- flag )
 */
void quarter_less_than(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, a < b ? -1 : 0);
}

/**
 * > - Greater than
 * Stack: ( a b -- flag )
 */
void quarter_gt(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, a > b ? -1 : 0);
}

/**
 * = - Equal
 * Stack: ( a b -- flag )
 */
void quarter_equal(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t b = pop_cell(memory, sp);
    int64_t a = pop_cell(memory, sp);
    push_cell(memory, sp, a == b ? -1 : 0);
}

// ============================================================================
// Memory Primitives
// ============================================================================

/**
 * @ - Fetch cell from memory
 * Stack: ( addr -- value )
 */
void quarter_fetch(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t addr = pop_cell(memory, sp);
    if (addr < 0 || addr >= MEMORY_SIZE - 8) {
        fprintf(stderr, "Memory access out of bounds: %lld\n", (long long)addr);
        exit(1);
    }
    int64_t value = *(int64_t*)(memory + addr);
    push_cell(memory, sp, value);
}

/**
 * ! - Store cell to memory
 * Stack: ( value addr -- )
 */
void quarter_store(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t addr = pop_cell(memory, sp);
    int64_t value = pop_cell(memory, sp);
    if (addr < 0 || addr >= MEMORY_SIZE - 8) {
        fprintf(stderr, "Memory access out of bounds: %lld\n", (long long)addr);
        exit(1);
    }
    *(int64_t*)(memory + addr) = value;
}

/**
 * C@ - Fetch byte from memory
 * Stack: ( addr -- byte )
 */
void quarter_c_fetch(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t addr = pop_cell(memory, sp);
    if (addr < 0 || addr >= MEMORY_SIZE) {
        fprintf(stderr, "Memory access out of bounds: %lld\n", (long long)addr);
        exit(1);
    }
    push_cell(memory, sp, (int64_t)memory[addr]);
}

/**
 * C! - Store byte to memory
 * Stack: ( byte addr -- )
 */
void quarter_c_store(uint8_t* memory, size_t* sp, size_t* rp) {
    (void)rp;
    int64_t addr = pop_cell(memory, sp);
    int64_t value = pop_cell(memory, sp);
    if (addr < 0 || addr >= MEMORY_SIZE) {
        fprintf(stderr, "Memory access out of bounds: %lld\n", (long long)addr);
        exit(1);
    }
    memory[addr] = (uint8_t)(value & 0xFF);
}
