ENTRY(boot)


SECTIONS {
    . = 0x80200000;

    .text :{
        KEEP(*(.text.boot));
        *(.text .text.*);
    }

    .rodata : ALIGN(4) {
        *(.rodata .rodata.*);
    }

    .data : ALIGN(4) {
        *(.data .data.*);
    }

    .bss : ALIGN(4) {
        __bss = .;
        *(.bss .bss.* .sbss .sbss.*);
        __bss_end = .;
    }

    .eh_frame :{
        *(.eh_frame);
    }

    /* this stack is used on entry and when initializing modules inside the kernel (everything except traps) */
    . = ALIGN(4);
    . += 128 * 1024; /* 128KB */
    __stack_top = .;

    /* this stack is used when handling a trap inside the kernel */
    . = ALIGN(4);
    . += 128 * 1024; /* 128KB */
    __kstack = .;

    /* this is used to store the trapframe before entering the trap handler */
    . = ALIGN(4);
    __trap_frame = .;
    . += 512;
    __trap_frame_end = .;

    . = ALIGN(4096);
    __ram = .;
    . += 64 * 1024 * 1024; /* 64MB */
    __ram_end = .;

    /* this is used for storing the virtio queue */
    . = ALIGN(4K);
    __virtq = .;
}


