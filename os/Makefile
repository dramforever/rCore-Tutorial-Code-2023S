# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_A := target/$(TARGET)/$(MODE)/libos.a
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin/*

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# GUI
GUI ?= off
ifeq ($(GUI), off)
	GUI_OPTION := -display none
endif

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
CC := riscv64-unknown-linux-gnu-gcc
LD := rust-lld -flavor gnu
STRIP := rust-strip
rvind := rvind-analyze

CHAPTER ?= $(shell git rev-parse --abbrev-ref HEAD | grep -oP 'ch\K[0-9]')
TEST ?= $(CHAPTER)
BASE ?= 1

# Disassembly
DISASM ?= -x

# Run usertests or usershell
TEST ?=

build: env $(KERNEL_BIN) fs-img

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install --git https://github.com/dramforever/rvind.git rvind-analyze
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): $(KERNEL_ELF)
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@make -C ../user build TEST=$(TEST) CHAPTER=$(CHAPTER) BASE=$(BASE)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/

$(KERNEL_ELF): kernel
	$(LD) -Tsrc/linker.ld --gc-sections --whole-archive -o $@.1 $(KERNEL_A)
	$(rvind) -o rvind.bin $@.1 > /dev/null
	$(CC) -c -o $@.rvind.o src/rvind_bin.s
	$(LD) -Tsrc/linker.ld --gc-sections --whole-archive -o $@ $(KERNEL_A) $@.rvind.o
	$(STRIP) -X $@
	$(rvind) -o rvind.bin.2 $@ > /dev/null
	cmp rvind.bin rvind.bin.2
	cmp rvind.bin.sym rvind.bin.2.sym
	cmp rvind.bin.str rvind.bin.2.str
	@rm -f rvind.bin*

kernel:
	@make -C ../user build TEST=$(TEST) CHAPTER=$(CHAPTER) BASE=$(BASE)
	@echo Platform: $(BOARD)
	@cargo build -Zbuild-std=core,alloc --release

clean:
	@cargo clean

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: build run-inner

run-inner-none: build
	@qemu-system-riscv64 \
		-M 128m \
		-machine virt \
		-bios none \
		$(GUI_OPTION) \
		-kernel $(KERNEL_ELF) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0 \
#		-device virtio-gpu-device  \
		-device virtio-keyboard-device  \
		-device virtio-mouse-device \
		-device virtio-net-device,netdev=net0 \
		-netdev user,id=net0,hostfwd=udp::6200-:2000 \
		-serial stdio

run-inner: build
	@qemu-system-riscv64 \
		-M 128m \
		-machine virt \
		-bios $(BOOTLOADER) \
		$(GUI_OPTION) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0 \
		-device virtio-gpu-device  \
		-device virtio-keyboard-device  \
		-device virtio-mouse-device \
		-device virtio-net-device,netdev=net0 \
		-netdev user,id=net0,hostfwd=udp::6200-:2000 \
		-serial stdio

fdt:
	@qemu-system-riscv64 -M 128m -machine virt,dumpdtb=virt.out
	fdtdump virt.out

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d


gdbserver: build
	@qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

.PHONY: build env kernel clean disasm disasm-vim run-inner fs-img gdbserver gdbclient fdt
