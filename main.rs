#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(panic_info_message)]
#![feature(step_trait_ext)]

extern crate alloc;
extern crate wee_alloc;

use alloc::boxed::Box;
// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use multiboot2::load;
use spin::Mutex;
use vga::colors::Color16;

mod asm;
mod ata;
mod gdt;
mod graphics;
mod idt;
mod keyboard;
mod mem;
mod mouse;
mod pci;
mod pic;
mod pspeaker;
mod rdtsc;
mod rtc;

use asm::*;
use graphics::Screen;
use keyboard::scancode_to_input;
use keyboard::Input;
use mouse::disable_mouse;
use rdtsc::rdtsc;

lazy_static! {
    pub static ref SCREEN: Mutex<Screen> = Mutex::new(Screen::new());
}

/// TSC timestamp at boot time stored here.
static mut BOOT_TICKS: f64 = 0.0;

// TODO: Implement `cowsay` once we have a userspace / command centre
static WELCOME: &[u8] = br#" ____________________
< rkernel is da best >
 --------------------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
"#;

// TODO: Maybe rename to print
#[macro_export]
macro_rules! raw_write {
    ($m: expr) => {
        crate::SCREEN.lock().write($m, vga::colors::Color16::White);
    };
}

#[macro_export]
macro_rules! log {
    ($msg: expr) => {
        crate::raw_write!(b"[");
        let dt = crate::rdtsc::delta_ns(unsafe { crate::BOOT_TICKS });
        let mut buffer = ryu::Buffer::new();
        let printable = buffer.format(dt);
        crate::SCREEN
            .lock()
            .write(&printable.as_bytes(), vga::colors::Color16::LightGreen);
        crate::SCREEN
            .lock()
            .write(b"ns", vga::colors::Color16::LightGreen);
        crate::raw_write!(b"] ");
        crate::raw_write!($msg);
    };
}

#[no_mangle]
pub extern "C" fn _start(m_ptr: usize) -> ! {
    unsafe { BOOT_TICKS = rdtsc() };
    let boot_info = unsafe { load(m_ptr) };
    let a = Box::new(10);
    log!(b"Enter _start\n");
    log!(b"TSC calibrated\n");
    mem::info(&boot_info);
    mouse::init();
    log!(b"Mouse enabled\n");
    ata::init();
    pci::init();
    idt::init();
    log!(b"Interrupts enabled\n");
    unsafe { pspeaker::beep(1000) };
    raw_write!(WELCOME);

    // TODO: use `hlt` instruction
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    raw_write!(alloc::format!("{}", info).as_bytes());
    // TODO: ACPI/APM shutdown or QEMU exit after few seconds.
    loop {}
}

#[alloc_error_handler]
#[no_mangle]
pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
    raw_write!(b"OOM");
    unsafe {
        ::core::intrinsics::abort();
    }
}
