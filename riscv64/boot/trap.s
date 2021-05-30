.altmacro
.set REG_SIZE, 8 

.macro save_gp i, basereg=t6
	sd	x\i, ((\i)*REG_SIZE)(\basereg)
.endm

.macro load_gp i, basereg=t6
	ld	x\i, ((\i)*REG_SIZE)(\basereg)
.endm

.section .text
.align 4
.global rupt

rupt:
    csrrw	t6, mscratch, t6
    
    .set 	i, 0
	  .rept	31
		  save_gp	%i
		  .set	i, i+1
	  .endr
    
    mv		t5, t6
	  csrr	t6, mscratch
	  save_gp 31, t5
    csrw	mscratch, t5

    csrr a0, mcause
    csrr a1, mepc
    csrr a2, mtval
    csrr a3, mscratch

    # ld sp, 248(a3)
    call _rupt

    csrw mepc, a0
    csrr t6, mscratch
    
    .set	i, 1
	  .rept	31
		  load_gp %i
		  .set	i, i+1
	  .endr

    mret
