pub fn init() {
    let mut t: usize;

    unsafe {
        asm!("clts", options(nostack));
        asm!("mov cr0, {}", out(reg) t, options(nostack));
        t &= !(1 << 2);
        t |= (1 << 1);
        asm!("mov {}, cr0", in(reg) t, options(nostack));
        asm!("mov cr4, {}", out(reg) t, options(nostack));
        t |= 3 << 9;
        asm!("mov {}, cr4", in(reg) t, options(nostack));
        asm!("fninit", options(nostack));
    }
}
