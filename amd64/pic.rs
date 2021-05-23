// Heavily adapted from https://github.com/emk/toyos-rs/blob/master/crates/pic8259_simple/src/lib.rs

use crate::asm::*;

pub struct Pic {
    offset: u8,
    cmd_port: u16,
    data_port: u16,
}

fn iowait() {
    unsafe { write_to_port(0x80, 0) };
}

impl Pic {
    pub const fn new(offset: u8, cmd_port: u16, data_port: u16) -> Self {
        Self {
            offset,
            cmd_port,
            data_port,
        }
    }

    fn end(&self) {
        unsafe {
            write_to_port(self.cmd_port, 0x20);
        } // End of interrupt
    }

    fn handle(&self, id: u8) -> bool {
        self.offset <= id && id < self.offset + 8
    }

    // Read data port
    fn read(&self) -> u8 {
        unsafe { read_from_port(self.data_port) }
    }
}

// Pair of Pics.
pub struct Chained(Pic, Pic);

impl Chained {
    pub const fn new(o1: u8, o2: u8) -> Self {
        Self(Pic::new(o1, 0x20, 0x21), Pic::new(o2, 0xA0, 0xA1))
    }

    pub unsafe fn init(&self) {
        let mask0 = self.0.read();
        let mask1 = self.1.read();

        write_to_port(self.0.cmd_port, 0x11);
        iowait();
        write_to_port(self.1.cmd_port, 0x11);
        iowait();

        write_to_port(self.0.data_port, self.0.offset);
        iowait();
        write_to_port(self.1.data_port, self.1.offset);
        iowait();

        write_to_port(self.0.data_port, 4);
        iowait();
        write_to_port(self.1.data_port, 2);
        iowait();

        write_to_port(self.0.data_port, 0x01);
        iowait();
        write_to_port(self.1.data_port, 0x01);
        iowait();

        write_to_port(self.0.data_port, mask0);
        write_to_port(self.1.data_port, mask1);
    }

    pub fn handle(&self, id: u8) -> bool {
        self.0.handle(id) || self.1.handle(id)
    }

    pub unsafe fn end(&self, id: u8) {
        if self.handle(id) {
            if self.1.handle(id) {
                self.1.end();
            }
            self.0.end();
        }
    }
}
