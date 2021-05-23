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

fn bcd_to_binary(bcd: u8) -> u8 {
    (bcd / 16) * 10 + (bcd & 0xf)
}

pub fn time() {
    while is_updating() {}

    let mode = unsafe { read_rtc(0x0B) };
    let mut s = unsafe { read_rtc(0x00) };
    let mut m = unsafe { read_rtc(0x02) };
    let mut h = unsafe { read_rtc(0x04) };
    let mut d = unsafe { read_rtc(0x07) };
    let mut month = unsafe { read_rtc(0x08) };
    let mut year = unsafe { read_rtc(0x09) };

    s = bcd_to_binary(s);
    m = bcd_to_binary(m);
    let is_pm = h & 0x80 == 0;
    if is_pm {
        h &= 0x7f;
    }
    h = bcd_to_binary(h);
    if is_pm {
        h += 12;
    }
    d = bcd_to_binary(d);
    month = bcd_to_binary(month);
    year = bcd_to_binary(year);

    let year = 2000 + year as isize;
    crate::log!(alloc::format!(
        "RTC: year: {}, month: {}, day: {}, hour: {}, minute: {}, second: {} \n",
        year,
        month,
        d,
        h,
        m,
        s
    )
    .as_bytes());
}
