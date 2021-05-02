#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use vga::colors::Color16;

mod asm;
mod graphics;
mod keyboard;
mod mouse;
mod rtc;

use asm::*;
use graphics::Screen;
use keyboard::scancode_to_input;
use keyboard::Input;
use mouse::disable_mouse;
use mouse::init_mouse;

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
macro_rules! write {
    ($s: expr, $m: expr) => {
        $s.write($m, Color16::White);
    };
}

macro_rules! log {
    ($s: expr, $t: expr, $msg: expr) => {
        write!($s, b"[");
        let dt = (rdtsc() - $t - 10f64) * 2300000000000f64;
        let mut buffer = ryu::Buffer::new();
        let printed = buffer.format(dt);
        $s.write(&printed.as_bytes()[0..6], Color16::LightGreen);
        $s.write(b"ns", Color16::LightGreen);
        write!($s, b"] ");
        write!($s, $msg);
    };
}

fn rdtsc() -> f64 {
    unsafe {
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc() as f64
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut screen = Screen::new();
    // Maybe we could add mouse functionality in future by a user toggle.
    // let mut is_mouse = false;
    write!(screen, WELCOME);
    let start = rdtsc();

    log!(screen, start, b"Boot successful");

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

                        write!(screen, &cmd);
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
    write!(screen, b"kernel panic");
    loop {}
}
