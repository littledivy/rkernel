use crate::asm::*;
use bit_field::BitField;

fn wait_bsy() {
    while unsafe { read_from_port(0x80) } & 0x80 == 0 {}
}

fn wait_drq() {
    while unsafe { read_from_port(0x08) } & 0x40 != 0 {}
}

pub unsafe fn read(lba: u32, sector: u8, buf: &mut [u8]) {
    wait_bsy();

    write_to_port(0x1F6, 0xE0 | ((lba >> 24) & 0xF) as u8);
    write_to_port(0x1F2, sector);
    write_to_port(0x1F3, lba as u8);
    write_to_port(0x1F4, (lba >> 8) as u8);
    write_to_port(0x1F5, (lba >> 16) as u8);

    // READ Command
    write_to_port(0x1F7, 0x20);

    for i in 0..sector {
        wait_bsy();
        wait_drq();

        for j in 0..256 {
            let data = readu16_from_port(0x1F0);
            buf[j * 2] = data.get_bits(0..8) as u8;
            buf[j * 2 + 1] = data.get_bits(8..16) as u8;
        }
    }
}

pub unsafe fn prune(drive: u8) {
    write_to_port(0x1F0 + 6, 0xA0 | (drive << 4)); // Select drive

    write_to_port(0x1F0 + 7, 0xEC); // IDENTIFY Command

    if read_from_port(0x1F0 + 7) != 0 {
        if read_from_port(0x1F0 + 3) == 0 || read_from_port(0x1F0 + 4) == 0 {
            // Check LB1 and LB2
            for i in 0..256 {
                if read_from_port(0x1F0 + 7).get_bit(6) {
                    let mut res = [0; 256];
                    for i in 0..256 {
                        let data = readu16_from_port(0x1F0);
                        res[i] = data;
                    }

                    let mut model = alloc::string::String::new();
                    for &r in res.get(27..47).unwrap() {
                        for &b in r.to_be_bytes().iter() {
                            model.push(b as char);
                        }
                    }
                    let bytes = ((res[61] as u32) << 16 | (res[60] as u32)) * 512;

                    crate::log!(
                        alloc::format!("ATA drive: {} ({} bytes)\n", model.trim(), bytes)
                            .as_bytes()
                    );
                    break;
                }
            }
        }
    }
}

// TODO: Anti-panic
pub fn init() {
    for d in 0..2 {
        unsafe {
            prune(d);
        }
    }
}
