//! The panic handler and backtrace

use crate::sbi::shutdown;
// use crate::task::current_kstack_top;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    "
        .weak rvind_data_start
        .weak rvind_data_end
        .weak rvind_sym_start
        .weak rvind_sym_end
        .weak rvind_str_start
        .weak rvind_str_end

        .section .rodata
        .p2align 3
    rvind_header:
        .quad rvind_data_start
        .quad rvind_data_end - rvind_data_start

    rvind_sym_start_addr:
        .quad rvind_sym_start
    rvind_sym_end_addr:
        .quad rvind_sym_end
    rvind_str_start_addr:
        .quad rvind_str_start
    rvind_str_end_addr:
        .quad rvind_str_end

        .previous
    "
);

#[derive(Debug)]
#[repr(C)]
struct SymbolEntry {
    code_offset: u32,
    str_offset: u32,
}

extern "C" {
    static rvind_header: rvind_unwinder::Header;
    static stext: u8;
    static rvind_sym_start_addr: *const SymbolEntry;
    static rvind_sym_end_addr: *const SymbolEntry;
    static rvind_str_start_addr: *const u8;
    static rvind_str_end_addr: *const u8;
}

fn symbolize(pc: usize) -> Option<(&'static [u8], u32)> {
    if unsafe { rvind_sym_start_addr }.is_null() {
        return None;
    }

    let text_start = unsafe { &stext } as *const u8 as usize;

    let offset: u32 = pc.checked_sub(text_start)?.try_into().ok()?;

    let symtab: &'static [SymbolEntry] = unsafe {
        core::slice::from_raw_parts(
            rvind_sym_start_addr,
            rvind_sym_end_addr.offset_from(rvind_sym_start_addr) as usize,
        )
    };

    let strtab: &'static [u8] = unsafe {
        core::slice::from_raw_parts(
            rvind_str_start_addr,
            rvind_str_end_addr.offset_from(rvind_str_start_addr) as usize,
        )
    };

    let index = symtab
        .partition_point(|sym| sym.code_offset <= offset)
        .checked_sub(1)?;
    let symbol = symtab.get(index)?;

    let str = &strtab[symbol.str_offset as usize..];
    let str_end = str.iter().position(|&x| x == 0)?;

    Some((&str[..str_end], offset - symbol.code_offset))
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
                if let Some((symbol, inner_offset)) = symbolize(frame.pc) {
                    println!(
                        " ({:#010x}) {} + {:#x}",
                        frame.pc,
                        symbol.escape_ascii(),
                        inner_offset,
                    );
                } else {
                    println!(" ({:#010x}) ???", frame.pc);
                }
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
