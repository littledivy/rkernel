use crate::asm::*;
use crate::idt::PICS;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::instructions::port::PortRead;
use x86_64::instructions::port::PortReadOnly;
use x86_64::structures::idt::InterruptStackFrame;

const SB_RESET: u16 = 0x226;
const SB_READ: u16 = 0x22A;
const SB_READ_STATUS: u16 = 0x22E;
const SB_WRITE: u16 = 0x22C;

lazy_static! {
    pub static ref SB16: Mutex<SoundBlaster> = Mutex::new(SoundBlaster::new());
}

static mut BUF: &[u8; 882] = &[32; 882];

pub fn init() -> (u8, u8) {
    unsafe {
        SB16.lock().reset();
        SB16.lock().init()
    }
}

pub struct SoundBlaster {
    read_port: Port<u8>,
    write_port: Port<u8>,
    reset_port: Port<u8>,
    read_status_port: PortReadOnly<u8>,
}

impl SoundBlaster {
    pub fn new() -> Self {
        Self {
            read_port: Port::new(SB_READ),
            write_port: Port::new(SB_WRITE),
            reset_port: Port::new(SB_READ),
            read_status_port: PortReadOnly::new(SB_READ_STATUS),
        }
    }

    pub unsafe fn init(&mut self) -> (u8, u8) {
        self.write_port.read();
        self.write_port.write(0xE1);

        // (Version Major, Version Minor)
        let version = (self.read_port.read(), self.read_port.read());

        self.set_sample_rate(22050);
        self._write(0xb0);
        self._write(0x10 | 0x00);

        let buffer_size = 22050 * (40 / 1000);
        let sample_count = (buffer_size / 2) - 1;

        self._write(((sample_count >> 0) & 0xFF) as u8);
        self._write(((sample_count >> 8) & 0xFF) as u8);

        self._write(0xd1);
        self._write(0xd6);

        write_to_port(0x224, 0x80);
        write_to_port(0x225, 0b10);
        self.transfer(BUF as *const u8, 882);
        version
    }

    pub unsafe fn reset(&mut self) -> bool {
        self.reset_port.write(1);
        self.reset_port.write(0);

        self.read_port.read() == 0xAA
    }

    unsafe fn _write(&mut self, byte: u8) {
        while self.write_port.read() & 0x80 != 0 {}
        self.write_port.write(byte);
    }

    unsafe fn _read(&mut self, byte: u8) {
        while self.read_status_port.read() & 0x80 == 0 {}
        self.read_port.write(byte);
    }

    pub unsafe fn set_sample_rate(&mut self, freq: u16) {
        self._write(0x41);
        self._write(((freq >> 8) & 0xFF) as u8);
        self._write((freq & 0xFF) as u8);
    }

    pub unsafe fn transfer(&mut self, buf: *const u8, len: u32) {
        let mode = 0x48;
        let channel = 5;

        write_to_port(0xd4, 4 + (channel % 4));
        write_to_port(0xd8, 1);
        write_to_port(0xd6, (channel % 4) | mode | (1 << 4));

        let offset: u16 = ((buf as usize / 2) % 65536) as u16;

        write_to_port(0xc4, (offset >> 0 & 0xFF) as u8);
        write_to_port(0xc4, (offset >> 8 & 0xFF) as u8);

        write_to_port(0xc6, ((len - 1) & 0xFF) as u8);
        write_to_port(0xc6, (((len - 1) >> 8) & 0xFF) as u8);

        write_to_port(0x8b, (buf as usize >> 16) as u8);

        write_to_port(0xd4, channel % 4);
    }
}

pub extern "x86-interrupt" fn sound_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        SB16.lock()._write(0xd5);
        read_from_port(0x22F);
        PICS.lock().end(32 + 5);
    }
}
