#![no_std]
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
mod cpuid;
mod fpu;
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
mod sb16;
mod serial;

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
        #[allow(unused_unsafe)]
        let boot_t = unsafe { crate::BOOT_TICKS };
        let dt = crate::rdtsc::delta_ns(boot_t);
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
    log!(b"Enter _start\n");
    log!(b"TSC calibrated\n");
    mem::info(&boot_info);
    cpuid::init();
    mouse::init();
    log!(b"Mouse enabled\n");
    ata::init();
    pci::init();
    // FIXME: FPU initalization sometimes crashes.
    // fpu::init();
    idt::init();
    rtc::time();
    log!(b"Interrupts enabled\n");

    // Play the 3rd Octave :D
    // for note in 0..7 {
    //    pspeaker::play_note(3, note);
    // }

    raw_write!(WELCOME);

    let (v_major, v_minor) = sb16::init();
    log!(alloc::format!("SB16: {} {}\n", v_major, v_minor).as_bytes());

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
#[allow(improper_ctypes_definitions)]
pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
    raw_write!(b"OOM");
    unsafe {
        ::core::intrinsics::abort();
    }
}
