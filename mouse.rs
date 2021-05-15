//! Based on a much readable implementation: https://github.com/dthain/basekernel/blob/master/kernel/mouse.c

use crate::asm::*;
use crate::idt::PICS;
use crate::SCREEN;
use lazy_static::lazy_static;
use ps2_mouse::Mouse;
use ps2_mouse::MouseState;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

pub struct State(i16, i16);

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
    pub static ref MOUSE_STATE: Mutex<State> = Mutex::new(State(0, 0));
}

pub fn init() {
    MOUSE.lock().init();
    MOUSE.lock().set_on_complete(on_complete);

    // Old code. Essentially the same as `ps2_mouse::Mouse::init()`
    // unsafe {
    //   write_to_port(0x64, 0x20);
    //   let status = read_from_port(0x64) | 0x02;
    //   write_to_port(0x64, 0x60);
    //   write_to_port(0x60, status & 0xDF);
    //   write_to_port(0x64, 0xD4);
    //   write_to_port(0x60, 0xF6); // Set defaults
    //   write_to_port(0x64, 0xD4);
    //   write_to_port(0x60, 0xF4); // Enable packet streaming
    // }
}

fn on_complete(mouse_state: MouseState) {
    let mut state = MOUSE_STATE.lock();
    let dx = mouse_state.get_x();
    let dy = mouse_state.get_y();
    state.0 += dx;
    state.1 -= dy;
    if state.0.is_negative() {
        state.0 = 0;
    }
    if state.1.is_negative() {
        state.1 = 0;
    }
    if state.0 >= 640 {
        state.0 = 640;
    }
    if state.1 >= 480 {
        state.1 = 480;
    }
    SCREEN.lock().restore_pointer();
    SCREEN.lock().set_mouse(state.0 as usize, state.1 as usize);
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let packet = read_from_port(0x60);
        MOUSE.lock().process_packet(packet);
        PICS.lock().end(44);
    }
}

// TODO: Is this needed?
pub fn disable_mouse() {
    unsafe {
        // write_to_port(0x64, 0xD4);
        write_to_port(0x64, 0xA7); // Disable mouse
    }
}
