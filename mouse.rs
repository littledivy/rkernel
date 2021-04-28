use crate::asm::*;

pub unsafe fn init_mouse() {
    write_to_port(0x64, 0x20);

    let status = read_from_port(0x64) | 0x02;
    write_to_port(0x64, 0x60);
    write_to_port(0x60, status & 0xDF);

    write_to_port(0x64, 0xD4);
    write_to_port(0x60, 0xF6); // Set defaults

    write_to_port(0x64, 0xD4);
    write_to_port(0x60, 0xF4); // Enable packet streaming
}
