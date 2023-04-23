    .global rvind_data_start
    .global rvind_data_end

    .section .rodata
    .p2align 2
rvind_data_start:
    .incbin "rvind.bin"
rvind_data_end:
