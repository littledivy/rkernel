#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
mod asm;
mod graphics;
mod keyboard;
mod mouse;

use asm::*;
use graphics::Screen;
use keyboard::scancode_to_ascii;
use mouse::disable_mouse;
use mouse::init_mouse;

static WELCOME: &[u8] = b"welcome to rkernel";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut screen = Screen::new();
    let mut is_mouse = false;
    screen.write(WELCOME);
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
            // Toggle mouse
            if c == 0x1d {
                is_mouse = if is_mouse {
                    disable_mouse();
                    false
                } else {
                    true
                };
                init_mouse();
                continue;
            }

            if is_mouse {
                screen.write_byte(c as u8);
            } else {
                // Backspace
                if c == 0x0E {
                    screen.pop();
                } else if let Some(ch) = scancode_to_ascii(c) {
                    screen.write_byte(ch);
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut screen = Screen::new();
    screen.write(b"kernel panic");
    loop {}
}
