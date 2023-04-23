//! The panic handler and backtrace

use crate::sbi::shutdown;
// use crate::task::current_kstack_top;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    "
        .weak rvind_data_start
        .weak rvind_data_end

        .section .rodata
        .p2align 3
    rvind_header:
        .quad rvind_data_start
        .quad rvind_data_end - rvind_data_start

        .previous
    "
);

extern "C" {
    static rvind_header: rvind_unwinder::Header;
    static stext: u8;
}

extern "C" fn do_unwind(pc: usize, ra: usize, sp: usize, fp: usize) {
    unsafe {
        rvind_unwinder::unwind(
            &rvind_header,
            &rvind_unwinder::Context {
                text_start: &stext as *const u8 as usize,
            },
            rvind_unwinder::FirstFrame {
                ra,
                frame: rvind_unwinder::CallFrame { pc, sp, fp },
            },
            &mut |frame| {
                println!(" {:#x}", frame.pc);
            },
        );
    }
}

#[naked]
pub(crate) extern "C" fn stacktrace_start() {
    unsafe {
        asm!("
            auipc a0, 0
            mv a1, ra
            mv a2, sp
            mv a3, s0
            tail {go}
        ",
        go = sym do_unwind,
        options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    stacktrace_start();
    shutdown(255)
}
