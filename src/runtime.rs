//! Minimal runtime library for AOT-compiled Forth programs
//! Contains ONLY the quarter_* primitive functions needed by compiled code.
//! NO dependencies on LLVM, inkwell, or the rest of Quarter.

// Memory and stack constants
const DATA_STACK_END: usize = 0x020000;  // Data stack: 0-128KB

/// Check if stack pointer is valid for reading N bytes
#[inline]
unsafe fn check_sp_read(sp_val: usize, bytes_needed: usize) -> bool {
    sp_val >= bytes_needed && sp_val < DATA_STACK_END
}

/// Check if stack can grow by N bytes without overflow
#[inline]
unsafe fn check_sp_write(sp_val: usize, bytes_to_add: usize) -> bool {
    sp_val < DATA_STACK_END && sp_val + bytes_to_add <= DATA_STACK_END
}

/// Macro for binary operations (a b -- result)
macro_rules! binary_op {
    ($name:ident, $op:expr) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            _rp: *mut usize
        ) {
            unsafe {
                let sp_val = *sp;
                if !check_sp_read(sp_val, 16) {
                    return;
                }
                let addr_a = memory.add(sp_val - 16) as *mut i64;
                let addr_b = memory.add(sp_val - 8) as *const i64;
                let a = *addr_a;
                let b = *addr_b;
                *addr_a = $op(a, b);
                *sp = sp_val - 8;
            }
        }
    };
}

/// Macro for unary operations (a -- result)
macro_rules! unary_op {
    ($name:ident, $op:expr) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            memory: *mut u8,
            sp: *mut usize,
            _rp: *mut usize
        ) {
            unsafe {
                let sp_val = *sp;
                if !check_sp_read(sp_val, 8) {
                    return;
                }
                let addr = memory.add(sp_val - 8) as *mut i64;
                let a = *addr;
                *addr = $op(a);
            }
        }
    };
}

// ============================================================================
// ARITHMETIC OPERATIONS
// ============================================================================

binary_op!(quarter_add, |a, b| a + b);
binary_op!(quarter_sub, |a, b| a - b);
binary_op!(quarter_mul, |a, b| a * b);
binary_op!(quarter_div, |a, b| if b != 0 { a / b } else { 0 });
binary_op!(quarter_mod, |a, b| if b != 0 { a % b } else { 0 });

unary_op!(quarter_negate, |a: i64| -a);
unary_op!(quarter_abs, |a: i64| a.abs());
unary_op!(quarter_1plus, |a: i64| a + 1);
unary_op!(quarter_1minus, |a: i64| a - 1);
unary_op!(quarter_2star, |a: i64| a * 2);
unary_op!(quarter_2slash, |a: i64| a / 2);

binary_op!(quarter_min, |a, b| if a < b { a } else { b });
binary_op!(quarter_max, |a, b| if a > b { a } else { b });

// ============================================================================
// COMPARISON OPERATIONS
// ============================================================================

binary_op!(quarter_less_than, |a, b| if a < b { -1 } else { 0 });
binary_op!(quarter_gt, |a, b| if a > b { -1 } else { 0 });
binary_op!(quarter_equal, |a, b| if a == b { -1 } else { 0 });
binary_op!(quarter_not_equal, |a, b| if a != b { -1 } else { 0 });
binary_op!(quarter_less_equal, |a, b| if a <= b { -1 } else { 0 });
binary_op!(quarter_greater_equal, |a, b| if a >= b { -1 } else { 0 });

// ============================================================================
// BITWISE OPERATIONS
// ============================================================================

binary_op!(quarter_and, |a, b| a & b);
binary_op!(quarter_or, |a, b| a | b);
binary_op!(quarter_xor, |a, b| a ^ b);
unary_op!(quarter_invert, |a: i64| !a);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_lshift(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = a << (b as u32);
        *sp = sp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_rshift(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *const i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = a >> (b as u32);
        *sp = sp_val - 8;
    }
}

// ============================================================================
// STACK OPERATIONS
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_dup(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) || !check_sp_write(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *const i64;
        let val = *addr;
        let dest = memory.add(sp_val) as *mut i64;
        *dest = val;
        *sp = sp_val + 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_drop(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        *sp = sp_val - 8;
        let _ = memory;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_swap(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_a = memory.add(sp_val - 16) as *mut i64;
        let addr_b = memory.add(sp_val - 8) as *mut i64;
        let a = *addr_a;
        let b = *addr_b;
        *addr_a = b;
        *addr_b = a;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_over(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) || !check_sp_write(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 16) as *const i64;
        let val = *addr;
        let dest = memory.add(sp_val) as *mut i64;
        *dest = val;
        *sp = sp_val + 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_rot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 24) {
            return;
        }
        let addr_a = memory.add(sp_val - 24) as *mut i64;
        let addr_b = memory.add(sp_val - 16) as *mut i64;
        let addr_c = memory.add(sp_val - 8) as *mut i64;
        let a = *addr_a;
        let b = *addr_b;
        let c = *addr_c;
        *addr_a = b;
        *addr_b = c;
        *addr_c = a;
    }
}

// ============================================================================
// MEMORY OPERATIONS
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_val = memory.add(sp_val - 8) as *const i64;
        let data_val = memory.add(sp_val - 16) as *const i64;
        let addr = addr_val.read_unaligned() as usize;
        let data = data_val.read_unaligned();
        
        if addr + 8 <= 8 * 1024 * 1024 {
            let dest = memory.add(addr) as *mut i64;
            dest.write_unaligned(data);
        }
        *sp = sp_val - 16;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let addr = addr_ptr.read_unaligned() as usize;
        
        let value = if addr + 8 <= 8 * 1024 * 1024 {
            let src = memory.add(addr) as *const i64;
            src.read_unaligned()
        } else {
            0
        };
        
        let dest = memory.add(sp_val - 8) as *mut i64;
        dest.write_unaligned(value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_c_store(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 16) {
            return;
        }
        let addr_val = memory.add(sp_val - 8) as *const i64;
        let data_val = memory.add(sp_val - 16) as *const i64;
        let addr = addr_val.read_unaligned() as usize;
        let data = data_val.read_unaligned();
        
        if addr < 8 * 1024 * 1024 {
            let dest = memory.add(addr);
            *dest = (data & 0xFF) as u8;
        }
        *sp = sp_val - 16;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_c_fetch(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr_ptr = memory.add(sp_val - 8) as *const i64;
        let addr = addr_ptr.read_unaligned() as usize;
        
        let value = if addr < 8 * 1024 * 1024 {
            let src = memory.add(addr);
            *src as i64
        } else {
            0
        };
        
        let dest = memory.add(sp_val - 8) as *mut i64;
        dest.write_unaligned(value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_base(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        const BASE_ADDR: i64 = 0x7FFFF8;
        let sp_val = *sp;
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(BASE_ADDR);
        *sp = sp_val + 8;
    }
}

// ============================================================================
// RETURN STACK OPERATIONS
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_to_r(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let src = memory.add(sp_val - 8) as *const i64;
        let val = src.read_unaligned();
        let dest = memory.add(rp_val) as *mut i64;
        dest.write_unaligned(val);
        *sp = sp_val - 8;
        *rp = rp_val + 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_from_r(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;
        // Return stack starts at 0x010000, check for underflow
        if rp_val < 0x010000 + 8 {
            return;
        }
        let src = memory.add(rp_val - 8) as *const i64;
        let val = src.read_unaligned();
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(val);
        *sp = sp_val + 8;
        *rp = rp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_r_fetch(memory: *mut u8, sp: *mut usize, rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        let rp_val = *rp;
        // Return stack starts at 0x010000, check for underflow
        if rp_val < 0x010000 + 8 {
            return;
        }
        let src = memory.add(rp_val - 8) as *const i64;
        let val = src.read_unaligned();
        let dest = memory.add(sp_val) as *mut i64;
        dest.write_unaligned(val);
        *sp = sp_val + 8;
    }
}

// ============================================================================
// I/O OPERATIONS
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_dot(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *const i64;
        let val = addr.read_unaligned();
        
        // Simple integer printing - write to stdout
        unsafe extern "C" {
            fn printf(fmt: *const u8, ...) -> i32;
        }
        let fmt = b"%lld \0".as_ptr();
        printf(fmt, val);
        
        *sp = sp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_emit(memory: *mut u8, sp: *mut usize, _rp: *mut usize) {
    unsafe {
        let sp_val = *sp;
        if !check_sp_read(sp_val, 8) {
            return;
        }
        let addr = memory.add(sp_val - 8) as *const i64;
        let val = addr.read_unaligned();

        unsafe extern "C" {
            fn putchar(c: i32) -> i32;
        }
        putchar(val as i32);
        
        *sp = sp_val - 8;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_cr(_memory: *mut u8, _sp: *mut usize, _rp: *mut usize) {
    unsafe extern "C" {
        fn putchar(c: i32) -> i32;
    }
    unsafe {
        putchar(10); // newline
    }
}

// ============================================================================
// RUNTIME INITIALIZATION
// ============================================================================

unsafe extern "C" {
    fn calloc(nmemb: usize, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

static mut RUNTIME_MEMORY: *mut u8 = 0 as *mut u8;
static mut RUNTIME_SP: usize = 0;
static mut RUNTIME_RP: usize = 0x010000;  // Return stack starts at 64KB

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_runtime_init() {
    unsafe {
        // Allocate 8MB of zeroed memory using calloc
        RUNTIME_MEMORY = calloc(8 * 1024 * 1024, 1);

        // Initialize stack pointers
        RUNTIME_SP = 0;  // Data stack starts at 0
        RUNTIME_RP = 0x010000;  // Return stack starts at 64KB
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_runtime_cleanup() {
    unsafe {
        if !RUNTIME_MEMORY.is_null() {
            free(RUNTIME_MEMORY);
            RUNTIME_MEMORY = 0 as *mut u8;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quarter_runtime_get_state(
    memory: *mut *mut u8,
    sp: *mut *mut usize,
    rp: *mut *mut usize
) {
    unsafe {
        *memory = RUNTIME_MEMORY;
        *sp = &mut RUNTIME_SP as *mut usize;
        *rp = &mut RUNTIME_RP as *mut usize;
    }
}

