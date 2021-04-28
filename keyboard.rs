// https://github.com/emk/toyos-rs/blob/c3377fb8c1c92a8c042dd94ad9bfcd9a20470ff9/src/arch/x86_64/keyboard.rs#L114
pub fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    let idx = scancode as usize;
    match scancode {
        0x01..=0x0D => Some(b"\x1B1234567890-="[idx - 0x01]),
        0x0F..=0x1C => Some(b"\tqwertyuiop[]\r"[idx - 0x0F]),
        0x1E..=0x28 => Some(b"asdfghjkl;'"[idx - 0x1E]),
        0x2C..=0x35 => Some(b"zxcvbnm,./"[idx - 0x2C]),
        0x39 => Some(b' '),
        _ => None,
    }
}
