use x86_64::instructions::port::Port;
use x86_64::instructions::port::PortRead;
use x86_64::instructions::port::PortReadOnly;

pub struct SoundBlaster {
    read_port: PortReadOnly<u8>,
    write_port: Port<u8>,
    reset_port: Port<u8>,
}

impl SoundBlaster {
    pub fn new() -> Self {
        Self {
            read_port: PortReadOnly::new(0x22A),
            write_port: Port::new(0x22C),
            reset_port: Port::new(0x226),
        }
    }

    pub unsafe fn init(&mut self) -> (u8, u8) {
        self.write_port.read();
        self.write_port.write(0xE1);

        // (Version Major, Version Minor)
        (self.read_port.read(), self.read_port.read())
    }

    pub unsafe fn reset(&mut self) -> bool {
        self.reset_port.write(1);
        self.reset_port.write(0);

        self.read_port.read() == 0xAA
    }
}
