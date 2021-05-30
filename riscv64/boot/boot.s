.section .data
.option norvc

.section .text.proem

.global _start
_start:
    # read hart
    csrr t0, mhartid

    # setup stuff
    bnez t0, _loop
    
    csrw satp, zero

    # set up global pointer
    .option push
    .option norelax
    la gp, __global_pointer
    .option pop

    # store start and end of bss
    la a0, __bss_start
    la a1, __bss_end

    bgeu a0, a1, fill_bss_done

    # fill bss with 0s
    fill_bss_start:
        sd zero, (a0)
        addi a0, a0, 8
        bltu a0, a1, fill_bss_start
    fill_bss_done:

    # set stack pointer
    la sp, __stack_top
    li t0, (1 << 12) | (1 << 11) | (1 << 7) | (1 << 3)

    csrw mstatus, t0

    la t1, kmain
    csrw mepc, t1

    la t2, rupt
    csrw mtvec, t2

    # la t2, kframe
    # csrw mscratch, t2

    # enable interrupts:
    li t3, (1 << 11) | (1 << 7) | (1 << 3)
    csrw mie, t3

    la ra, _loop

    # jump to Rust
    mret

.global _loop

_loop:
    wfi
    j _loop

