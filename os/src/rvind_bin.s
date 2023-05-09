    .global rvind_data_start
    .global rvind_data_end
    .global rvind_sym_start
    .global rvind_sym_end
    .global rvind_str_start
    .global rvind_str_end

    .section .rodata
    .p2align 2
rvind_data_start:
    .incbin "rvind.bin"
rvind_data_end:

    .p2align 2
rvind_sym_start:
    .incbin "rvind.bin.sym"
rvind_sym_end:

rvind_str_start:
    .incbin "rvind.bin.str"
rvind_str_end:
