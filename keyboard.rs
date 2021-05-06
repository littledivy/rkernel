use crate::asm::*;
use crate::idt::PICS;
use crate::SCREEN;
use x86_64::structures::idt::InterruptStackFrame;

pub enum Input {
    /// A genuine character key.
    Key(u8),
    Enter,
    Esc,
}

// Based on https://github.com/emk/toyos-rs/blob/c3377fb8c1c92a8c042dd94ad9bfcd9a20470ff9/src/arch/x86_64/keyboard.rs#L114
pub fn scancode_to_input(scancode: u8) -> Option<Input> {
    let idx = scancode as usize;
    match scancode {
        0x01..=0x0D => Some(Input::Key(b"\x1B1234567890-="[idx - 0x01])),
        0x0F..=0x1B => Some(Input::Key(b"\tqwertyuiop[]"[idx - 0x0F])),
        0x1C => Some(Input::Enter),
        0x1E..=0x28 => Some(Input::Key(b"asdfghjkl;'"[idx - 0x1E])),
        0x2C..=0x35 => Some(Input::Key(b"zxcvbnm,./"[idx - 0x2C])),
        // TODO(littledivy): Make this unique in keyboard::Input ?
        0x39 => Some(Input::Key(b' ')),
        _ => None,
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // Get status from command port
        //  let status = read_from_port(0x64);

        // Data port is busy?
        //  if status & 0x1 != 0 {
        // Get input from data port
        let c = read_from_port(0x60);

        // Backspace
        if c == 0x0E {
            SCREEN.lock().pop();
        } else if let Some(input) = scancode_to_input(c) {
            match input {
                // Execute command
                Input::Enter => {
                    let cmd = SCREEN.lock().curr_command;
                    crate::raw_write!(&cmd);
                    SCREEN.lock().clear_command();
                }
                Input::Key(ch) => SCREEN.lock().write_byte(ch),
                _ => (),
            }
        }
        //}

        PICS.lock().end(33);
    }
}
