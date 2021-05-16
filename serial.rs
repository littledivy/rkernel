use crate::asm::*;
use x86_64::instructions::port::Port;

pub struct SerialPort(u16);

impl SerialPort {
    pub unsafe fn new(port: u16) -> Self {
        let mut p0 = Port::<u8>::new(port);
        let mut p1 = Port::<u8>::new(port + 1);
        let mut p2 = Port::<u8>::new(port + 2);
        let mut p3 = Port::<u8>::new(port + 3);
        let mut p4 = Port::<u8>::new(port + 4);

        // Disable intr
        p1.write(0x00);
        // Enable divisor mode
        p3.write(0x80);
        // Set port to 115200 bps
        p0.write(0x01);
        p1.write(0x00);
        // Disable divisor mode
        p3.write(0x03);
        // Clear FIFO
        p2.write(0xC7);
        // Enable intr
        p4.write(0x0B);
        p1.write(0x01);

        Self(port)
    }

    pub unsafe fn write(&self, byte: u8) {
        while read_from_port(self.0 + 5) & 0x20 == 0 {}
        write_to_port(self.0, byte);
    }

    pub unsafe fn read(&self) -> u8 {
        while read_from_port(self.0 + 5) & 1 == 0 {}
        read_from_port(self.0)
    }
}
