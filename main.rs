#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;

#[inline]
unsafe fn read_from_port(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

// https://github.com/emk/toyos-rs/blob/c3377fb8c1c92a8c042dd94ad9bfcd9a20470ff9/src/arch/x86_64/keyboard.rs#L114
fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    let idx = scancode as usize;
    match scancode {
        0x01...0x0E => Some(b"\x1B1234567890-=\0x02"[idx - 0x01]),
        0x0F...0x1C => Some(b"\tqwertyuiop[]\r"[idx - 0x0F]),
        0x1E...0x28 => Some(b"asdfghjkl;'"[idx - 0x1E]),
        0x2C...0x35 => Some(b"zxcvbnm,./"[idx - 0x2C]),
        0x39 => Some(b' '),
        _ => None,
    }
}

static HELLO: &[u8] = b"Hello World!";

pub struct Screen {
    pos: isize,
}

impl Screen {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    fn inc(&mut self) {
        self.pos += 1;
    }

    pub fn write(&mut self, buf: &[u8]) {
        let vga_buffer = 0xb8000 as *mut u8;

        for (i, byte) in buf.iter().enumerate() {
            self.write_byte(*byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        let vga_buffer = 0xb8000 as *mut u8;

        unsafe {
            self.inc();
            *vga_buffer.offset(self.pos * 2) = byte;
            *vga_buffer.offset(self.pos * 2 + 1) = 0xb;
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut screen = Screen::new();
    screen.write(HELLO);
    loop {
        unsafe {
            let status = read_from_port(0x64);
            if status & 0x1 == 0 {
                continue;
            }
            
            let c = read_from_port(0x60);
            
            if let Some(ch) = scancode_to_ascii(c) {
              screen.write_byte(ch);
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
