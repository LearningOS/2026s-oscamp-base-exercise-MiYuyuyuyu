//! # Stackful Coroutine and Context Switch (riscv64)
//!
//! In this exercise, you implement the minimal context switch using inline assembly,
//! which is the core mechanism of OS thread scheduling. This crate is **riscv64 only**;
//! run `cargo test` on riscv64 Linux, or use the repo's normal flow (`./check.sh` / `oscamp`) on x86 with QEMU.
//!
//! ## Key Concepts
//! - **Callee-saved registers**: Save and restore them on switch so the switched-away task can resume correctly later.
//! - **Stack pointer `sp`** and **return address `ra`**: Restore them in the new context; the first time we switch to a task, `ret` jumps to `ra` (the entry point).
//! - Inline assembly: `core::arch::asm!`
//!
//! ## riscv64 ABI (for this exercise)
//! - Callee-saved: `sp`, `ra`, `s0`–`s11`. The `ret` instruction is `jalr zero, 0(ra)`.
//! - First and second arguments: `a0` (old context), `a1` (new context).

#![cfg(target_arch = "riscv64")]

/// Saved register state for one task (riscv64). Layout must match the offsets used in the asm below: for one task (riscv64). Layout must match the offsets used in the asm below:
/// `sp` at 0, `ra` at 8, then `s0`–`s11` at 16, 24, … 104.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskContext {
    pub sp: u64,
    pub ra: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
}

impl TaskContext {
    pub const fn empty() -> Self {
        Self {
            sp: 0,
            ra: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }

    /// Initialize this context so that when we switch to it, execution starts at `entry`.
    ///
    /// - Set `ra = entry` so that the first `ret` in the new context jumps to `entry`.
    /// - Set `sp = stack_top` with 16-byte alignment (RISC-V ABI requires 16-byte aligned stack at function entry).
    /// - Leave `s0`–`s11` zero; they will be loaded on switch.
    pub fn init(&mut self, stack_top: usize, entry: usize) {
        // todo!("set ra = entry, sp = stack_top (16-byte aligned)")
        self.ra = entry as u64;
        
        // 设置栈指针，确保16字节对齐（RISC-V ABI要求）
        // 栈向下增长，所以stack_top应该是栈顶的高地址
        self.sp = (stack_top as u64) & !0xF; // 确保16字节对齐
    }
}

/// Switch from `old` to `new` context: save current callee-saved regs into `old`, load from `new`, then `ret` (jumps to `new.ra`).
///
/// In asm: store `sp`, `ra`, `s0`–`s11` to `[a0]` (old), load from `[a1]` (new), zero `a0`/`a1` so we do not leak pointers into the new context, then `ret`.
///
/// Must be `#[unsafe(naked)]` to prevent the compiler from generating a prologue/epilogue.
pub unsafe fn switch_context(old: &mut TaskContext, new: &TaskContext) {
    core::arch::asm!(
        // 保存当前上下文到old
        "sd sp, 0(a0)",     // 保存栈指针
        "sd ra, 8(a0)",     // 保存返回地址
        "sd s0, 16(a0)",    // 保存s0
        "sd s1, 24(a0)",    // 保存s1
        "sd s2, 32(a0)",    // 保存s2
        "sd s3, 40(a0)",    // 保存s3
        "sd s4, 48(a0)",    // 保存s4
        "sd s5, 56(a0)",    // 保存s5
        "sd s6, 64(a0)",    // 保存s6
        "sd s7, 72(a0)",    // 保存s7
        "sd s8, 80(a0)",    // 保存s8
        "sd s9, 88(a0)",    // 保存s9
        "sd s10, 96(a0)",   // 保存s10
        "sd s11, 104(a0)",  // 保存s11
        
        // 从new加载新上下文
        "ld sp, 0(a1)",     // 加载栈指针
        "ld ra, 8(a1)",     // 加载返回地址
        "ld s0, 16(a1)",    // 加载s0
        "ld s1, 24(a1)",    // 加载s1
        "ld s2, 32(a1)",    // 加载s2
        "ld s3, 40(a1)",    // 加载s3
        "ld s4, 48(a1)",    // 加载s4
        "ld s5, 56(a1)",    // 加载s5
        "ld s6, 64(a1)",    // 加载s6
        "ld s7, 72(a1)",    // 加载s7
        "ld s8, 80(a1)",    // 加载s8
        "ld s9, 88(a1)",    // 加载s9
        "ld s10, 96(a1)",   // 加载s10
        "ld s11, 104(a1)",  // 加载s11
        
        // 清除a0和a1，避免将旧上下文指针泄漏到新任务
        "mv a0, zero",
        "mv a1, zero",
        
        // 跳转到新上下文的返回地址（任务入口点）
        "ret",
        
        // 输入参数约束
        in("a0") old,  // 第一个参数：旧上下文指针
        in("a1") new,  // 第二个参数：新上下文指针
        options(noreturn)  // 函数不会返回
    );
    // todo!("save callee-saved regs to old, load from new, then ret; use #[unsafe(naked)] + naked_asm!, see module doc for riscv64 ABI and layout")
}

const STACK_SIZE: usize = 1024 * 64;

/// Allocate a stack for a coroutine. Returns `(buffer, stack_top)` where `stack_top` is the high address
/// (stack grows down). The buffer must be kept alive for the lifetime of the context using this stack.
pub fn alloc_stack() -> (Vec<u8>, usize) {
    // todo!("allocate stack buffer, return (buffer, stack_top) with stack_top 16-byte aligned")
    let mut buffer = vec![0u8; STACK_SIZE];
    
    // 获取栈顶地址（高地址，因为栈向下增长）
    let stack_top = buffer.as_ptr() as usize + STACK_SIZE;
    
    // 确保栈顶16字节对齐
    let aligned_top = stack_top & !0xF;
    
    (buffer, aligned_top)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    extern "C" fn task_entry() {
        COUNTER.store(42, Ordering::SeqCst);
        loop {
            std::hint::spin_loop();
        }
    }

    #[test]
    fn test_alloc_stack() {
        let (buf, top) = alloc_stack();
        assert_eq!(top, buf.as_ptr() as usize + STACK_SIZE);
        assert!(top % 16 == 0);
    }

    #[test]
    fn test_context_init() {
        let (buf, top) = alloc_stack();
        let _ = buf;
        let mut ctx = TaskContext::empty();
        let entry = task_entry as *const () as usize;
        ctx.init(top, entry);
        assert_eq!(ctx.ra, entry as u64);
        assert!(ctx.sp != 0);
    }

    #[test]
    fn test_switch_to_task() {
        COUNTER.store(0, Ordering::SeqCst);

        static mut MAIN_CTX_PTR: *mut TaskContext = std::ptr::null_mut();
        static mut TASK_CTX_PTR: *mut TaskContext = std::ptr::null_mut();

        extern "C" fn cooperative_task() {
            COUNTER.store(99, Ordering::SeqCst);
            unsafe {
                switch_context(&mut *TASK_CTX_PTR, &*MAIN_CTX_PTR);
            }
        }

        let (_stack_buf, stack_top) = alloc_stack();
        let mut main_ctx = TaskContext::empty();
        let mut task_ctx = TaskContext::empty();
        task_ctx.init(stack_top, cooperative_task as *const () as usize);

        unsafe {
            MAIN_CTX_PTR = &mut main_ctx;
            TASK_CTX_PTR = &mut task_ctx;
            switch_context(&mut main_ctx, &task_ctx);
        }

        assert_eq!(COUNTER.load(Ordering::SeqCst), 99);
    }
}
