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
