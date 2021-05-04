#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use multiboot2::load;
use vga::colors::Color16;

mod asm;
mod graphics;
mod keyboard;
mod mem;
mod mouse;
mod rdtsc;
mod rtc;

use asm::*;
use graphics::Screen;
use keyboard::scancode_to_input;
use keyboard::Input;
use mouse::disable_mouse;
use mouse::init_mouse;
use rdtsc::rdtsc;

static mut BOOT_TICKS: f64 = 0.0;

static WELCOME: &[u8] = br#" ____________________
< rkernel is da best >
 --------------------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
"#;

// Convinient macro to write white text on provided screen
#[macro_export]
macro_rules! raw_write {
    ($s: expr, $m: expr) => {
        $s.write($m, vga::colors::Color16::White);
    };
}

#[macro_export]
macro_rules! log {
    ($s: expr, $msg: expr) => {
        crate::raw_write!($s, b"[");
        let dt = crate::rdtsc::delta_ns(unsafe { crate::BOOT_TICKS });
        let mut buffer = ryu::Buffer::new();
        let printable = buffer.format(dt);
        $s.write(
            &printable.as_bytes()[0..6],
            vga::colors::Color16::LightGreen,
        );
        $s.write(b"ns", vga::colors::Color16::LightGreen);
        crate::raw_write!($s, b"] ");
        crate::raw_write!($s, $msg);
    };
}

#[no_mangle]
pub extern "C" fn _start(m_ptr: usize) -> ! {
    unsafe { BOOT_TICKS = rdtsc() };
    let boot_info = unsafe { load(m_ptr) };
    let mut screen = Screen::new();
    // Maybe we could add mouse functionality in future by a user toggle.
    // let mut is_mouse = false;
    //raw_write!(screen, WELCOME);
    let start = rdtsc();

    log!(screen, b"Boot successful\n");
    log!(screen, b"TSC calibrated\n");
    mem::info(&boot_info, &mut screen);
    loop {
        unsafe {
            // Keyboard things
            // Get status from command port
            let status = read_from_port(0x64);

            // Is the data port is busy?
            if status & 0x1 == 0 {
                continue;
            }
            // Get input from data port
            let c = read_from_port(0x60);

            // Backspace
            if c == 0x0E {
                screen.pop();
            } else if let Some(input) = scancode_to_input(c) {
                match input {
                    // Execute command
                    Input::Enter => {
                        let cmd = screen.curr_command;
                        raw_write!(screen, &cmd);
                        screen.clear_command();
                    }
                    Input::Key(ch) => screen.write_byte(ch),
                    _ => (),
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut screen = Screen::new();
    raw_write!(screen, b"kernel panic");
    loop {}
}
