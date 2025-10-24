# Current LLVM IR Analysis

This document shows what LLVM IR we're currently generating and identifies optimization opportunities for Phase 7.

## Current State: Function Call Overhead

### Example 1: SQUARE (DUP *)

**Forth Code:**
```forth
: SQUARE DUP * ;
```

**Current LLVM IR:**
```llvm
define void @SQUARE(ptr %0, ptr %1, ptr %2) {
entry:
  call void @quarter_dup(ptr %0, ptr %1, ptr %2)
  call void @quarter_mul(ptr %0, ptr %1, ptr %2)
  ret void
}
```

**Problem:** Two function calls to Rust functions instead of inline operations.

**Optimized IR (target):**
```llvm
define void @SQUARE(ptr %0, ptr %1, ptr %2) {
entry:
  ; Load stack pointer
  %sp = load i64, ptr %1, align 8

  ; Load top value
  %addr = getelementptr i8, ptr %0, i64 %sp
  %top_ptr = getelementptr i32, ptr %addr, i32 -1
  %top = load i32, ptr %top_ptr, align 4

  ; DUP: Store top value at new location
  store i32 %top, ptr %addr, align 4
  %new_sp = add i64 %sp, 4

  ; MUL: Load both values and multiply
  %addr2 = getelementptr i8, ptr %0, i64 %new_sp
  %b_ptr = getelementptr i32, ptr %addr2, i32 -1
  %a_ptr = getelementptr i32, ptr %addr2, i32 -2
  %a = load i32, ptr %a_ptr, align 4
  %b = load i32, ptr %b_ptr, align 4
  %result = mul i32 %a, %b

  ; Store result and update SP
  %result_ptr = getelementptr i32, ptr %addr2, i32 -2
  store i32 %result, ptr %result_ptr, align 4
  %final_sp = sub i64 %new_sp, 4
  store i64 %final_sp, ptr %1, align 8

  ret void
}
```

After LLVM optimization passes, this will be further simplified.

---

### Example 2: DOUBLE (DUP +)

**Forth Code:**
```forth
: DOUBLE DUP + ;
```

**Current LLVM IR:**
```llvm
define void @DOUBLE(ptr %0, ptr %1, ptr %2) {
entry:
  call void @quarter_dup(ptr %0, ptr %1, ptr %2)
  call void @quarter_add(ptr %0, ptr %1, ptr %2)
  ret void
}
```

**Problem:** Same issue - function calls instead of inline operations.

---

### Example 3: COMPLEX (5 DUP * 3 DUP * +)

**Forth Code:**
```forth
: COMPLEX 5 DUP * 3 DUP * + ;  \ 5^2 + 3^2 = 34
```

**Current LLVM IR:**
```llvm
define void @COMPLEX(ptr %0, ptr %1, ptr %2) {
entry:
  ; Push 5
  %sp = load i64, ptr %1, align 8
  %addr = getelementptr i8, ptr %0, i64 %sp
  store i32 5, ptr %addr, align 4
  %new_sp = add i64 %sp, 4
  store i64 %new_sp, ptr %1, align 8

  ; DUP and MUL (via function calls)
  call void @quarter_dup(ptr %0, ptr %1, ptr %2)
  call void @quarter_mul(ptr %0, ptr %1, ptr %2)

  ; Push 3
  %sp1 = load i64, ptr %1, align 8
  %addr2 = getelementptr i8, ptr %0, i64 %sp1
  store i32 3, ptr %addr2, align 4
  %new_sp3 = add i64 %sp1, 4
  store i64 %new_sp3, ptr %1, align 8

  ; DUP, MUL, and ADD (via function calls)
  call void @quarter_dup(ptr %0, ptr %1, ptr %2)
  call void @quarter_mul(ptr %0, ptr %1, ptr %2)
  call void @quarter_add(ptr %0, ptr %1, ptr %2)
  ret void
}
```

**Observations:**
- ✅ Literal numbers (5, 3) are already inlined
- ❌ Stack operations (DUP, *, +) are function calls
- ⚠️ Stack pointer loaded/stored multiple times (optimization opportunity)

---

## Phase 7 Optimization Strategy

### 1. Inline Primitives (High Priority)

Replace function calls with direct LLVM instructions:

| Forth Word | Current | Target LLVM |
|------------|---------|-------------|
| `+` | `call @quarter_add` | `add i32 %a, %b` |
| `-` | `call @quarter_sub` | `sub i32 %a, %b` |
| `*` | `call @quarter_mul` | `mul i32 %a, %b` |
| `/` | `call @quarter_div` | `sdiv i32 %a, %b` |
| `DUP` | `call @quarter_dup` | `load` + `store` inline |
| `SWAP` | `call @quarter_swap` | `load` + `load` + `store` + `store` |
| `DROP` | `call @quarter_drop` | adjust SP only |

### 2. Stack Pointer Optimization

**Current:** Load/store SP on every operation
**Target:** Keep SP in a register, use LLVM's `mem2reg` pass to optimize

### 3. Enable LLVM Optimization Passes

Add these passes in the JIT compiler:
- `mem2reg` - Promote memory to registers
- `instcombine` - Combine instructions
- `reassociate` - Reassociate expressions
- `gvn` - Global value numbering
- `cfg-simplification` - Simplify control flow
- `inline` - Inline function calls
- `dce` - Dead code elimination

### 4. Expected Results

For the COMPLEX example, after all optimizations:
```llvm
define void @COMPLEX(ptr %0, ptr %1, ptr %2) {
entry:
  ; 5^2 + 3^2 = 25 + 9 = 34 (constant folded!)
  %sp = load i64, ptr %1, align 8
  %addr = getelementptr i8, ptr %0, i64 %sp
  store i32 34, ptr %addr, align 4
  %new_sp = add i64 %sp, 4
  store i64 %new_sp, ptr %1, align 8
  ret void
}
```

The entire computation constant-folded to just pushing 34!

---

## Next Steps

1. Start with inlining `+` operation
2. Test with SQUARE benchmark
3. Add remaining primitives one by one
4. Enable optimization passes
5. Measure performance improvements
