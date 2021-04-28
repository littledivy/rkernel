use crate::asm::*;

unsafe fn read_rtc(address: u16) -> u8 {
    write_to_port(address, 0x70); // Write to ADDRESS port
    read_from_port(0x71) // Read from DATA port
}

pub unsafe fn read_time() {
    let s = read_rtc(0x80 + 0); // RTC Seconds
    let m = read_rtc(0x80 + 2); // RTC Minutes
    screen.write_byte((s & 0x0f) + (s >> 4) * 10);
}
