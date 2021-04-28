pub struct Screen {
    pos: isize,
}

impl Screen {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn pop(&mut self) {
        let vga_buffer = 0xb8000 as *mut u8;
        unsafe {
            *vga_buffer.offset(self.pos * 2 + 1) = 0xb;
            *vga_buffer.offset(self.pos * 2) = b' ';
        }
        self.pos -= 1;
    }

    pub fn write(&mut self, buf: &[u8]) {
        for (i, byte) in buf.iter().enumerate() {
            self.write_byte(*byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        let vga_buffer = 0xb8000 as *mut u8;

        unsafe {
            self.pos += 1;
            *vga_buffer.offset(self.pos * 2) = byte;
            *vga_buffer.offset(self.pos * 2 + 1) = 0xb;
        }
    }
}
