use crate::asm::*;

fn io_wait() {
    unsafe { write_to_port(0, 0x80) };
}

unsafe fn read_rtc(address: u8) -> u8 {
    write_to_port(0x70, address); // Write to ADDRESS port
    io_wait();
    let data = read_from_port(0x71); // Read from DATA port
    io_wait();
    data
}

fn is_updating() -> bool {
    unsafe {
        write_to_port(0x70, 0x0a);
        read_from_port(0x71) & 0x80 == 0
    }
}

pub fn read_time() -> u8 {
    while is_updating() {}
    let s = unsafe { read_rtc(0x00) }; // RTC Seconds
                                       // let m = unsafe { read_rtc(0x80 + 2); } // RTC Minutes
                                       // let h = unsafe { read_rtc(0x80 + 4); } // RTC Hour
    let mode = unsafe { read_rtc(0x0B) };
    if mode & 0x04 == 0 {
        ((s / 16) * 10 + (s & 0xf))
    } else {
        s
    }
}
