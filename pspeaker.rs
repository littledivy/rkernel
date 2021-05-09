use crate::asm::*;

// TODO: Set PIT back to it's initial frequency
pub unsafe fn beep(freq: u32) {
    let reload = 1193180 / freq;
    write_to_port(0x43, 0xb6);
    write_to_port(0x42, reload as u8);
    write_to_port(0x42, (reload >> 8) as u8);

    let status = read_from_port(0x61);
    if status != status | 3 {
        write_to_port(0x61, status | 3);
    }
    // beep for some time
    // TODO: eh there's a better way of waiting
    for _ in 0..1000000 {}
    let status = read_from_port(0x61) & 0xFC;
    write_to_port(0x61, status);
}
