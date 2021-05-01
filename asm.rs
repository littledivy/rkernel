/// This module provides I/O Port communication with some inline assembly *magic*.
/// Needs `#![feature(asm)]` (nightly rustc)

#[inline]
pub unsafe fn read_from_port(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

#[inline]
pub unsafe fn write_to_port(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}
