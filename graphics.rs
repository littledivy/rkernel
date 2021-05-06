/// `rkernel::graphics::Screen` provides the high level API to draw on the 640x480 screen with 16 layers.
use core::fmt;

use vga::colors::Color16;
use vga::colors::TextModeColor;
use vga::vga::VGA;
use vga::writers::{ScreenCharacter, Text80x25, TextWriter};

trait Writer {
    fn inc(&mut self) {}
    fn dec(&mut self) {}
}

struct CommandWriter {
    /// Only dynamic value of the command input.
    pub x: usize,
    /// Y remains constant at `WBOUNDARY - 10`.
    /// We refer this when drawing to screen.
    pub y: usize,
}

static CommandY: usize = 25 - 1;

impl CommandWriter {
    fn init() -> Self {
        Self { x: 1, y: CommandY }
    }
}

impl Writer for CommandWriter {
    fn inc(&mut self) {
        self.x += 1;
        if self.x > 80 {
            self.x = 1;
        }
    }

    fn dec(&mut self) {
        if self.x <= 1 {
            self.x = 1;
        } else {
            self.x -= 1;
        }
    }
}

struct StageWriter {
    pub x: usize,
    pub y: usize,
}

impl StageWriter {
    pub fn init() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Writer for StageWriter {
    fn inc(&mut self) {
        self.x += 1;
        if self.x > 80 {
            self.x = 0;
            self.y += 1;
        }
    }

    fn dec(&mut self) {
        if self.x < 0 {
            self.x = 0;
            if self.y < 1 {
                self.y = 1;
            } else {
                self.y -= 1;
            }
        } else {
            self.x -= 1;
        }
    }
}

struct MousePointer {
    x: usize,
    y: usize,
    preserved: Option<ScreenCharacter>,
}

impl Default for MousePointer {
    fn default() -> Self {
        // Default coordinates must match with `crate::mouse::State` defaults.
        Self {
            x: 0,
            y: 0,
            preserved: None,
        }
    }
}

/// Uh, so this is the screen. Not literally
pub struct Screen {
    pub mode: Text80x25,
    cmd: CommandWriter,
    pub curr_command: [u8; 310],
    stage: StageWriter,
    pointer: MousePointer,
}

impl Screen {
    /// Creates a new screen on top of the 640x480x16 VGA Graphics writer.
    pub fn new() -> Self {
        let mode = Text80x25::new();
        mode.set_mode();
        mode.clear_screen();
        let color = TextModeColor::new(Color16::White, Color16::Black);
        let marker = ScreenCharacter::new(b'_', color);
        for i in 0..80 {
            mode.write_character(i, 25 - 2, marker);
        }
        mode.write_character(0, 25 - 1, ScreenCharacter::new(b'>', color));
        Self {
            mode,
            cmd: CommandWriter::init(),
            stage: StageWriter::init(),
            pointer: Default::default(),
            // Allocate memory for command storage
            curr_command: [0u8; 310],
        }
    }

    /// Writes to the stage.
    pub fn write(&mut self, buf: &[u8], fg: Color16) {
        for (offset, ch) in buf.iter().enumerate() {
            if ch == &0u8 {
                continue;
            };
            if ch == &b'\n' {
                self.stage.x = 0;
                self.stage.y += 1;
            } else {
                let color = TextModeColor::new(fg, Color16::Black);
                let screen_character = ScreenCharacter::new(*ch, color);
                self.mode
                    .write_character(self.stage.x, self.stage.y, screen_character);
                self.stage.inc();
            }
        }
    }

    /// Writes to the command input.
    pub fn write_byte(&mut self, ch: u8) {
        let color = TextModeColor::new(Color16::Yellow, Color16::Black);
        let screen_character = ScreenCharacter::new(ch, color);
        self.mode.set_cursor_position(self.cmd.x, self.cmd.y);
        self.mode
            .write_character(self.cmd.x, self.cmd.y, screen_character);
        self.curr_command[self.cmd.x] = ch;
        self.cmd.inc();
    }

    /// Resets the command.
    pub fn clear_command(&mut self) {
        while self.cmd.x != 1 {
            self.pop();
        }
        self.pop();
        self.curr_command = [0u8; 310];
    }

    /// Resets the previous 8x8 character from R to L of the command input.
    pub fn pop(&mut self) {
        let color = TextModeColor::new(Color16::Black, Color16::Black);
        let blch = ScreenCharacter::new(b' ', color);
        self.mode.write_character(self.cmd.x, self.cmd.y, blch);
        self.cmd.dec();
        self.mode.set_cursor_position(self.cmd.x, self.cmd.y);
    }

    pub fn set_mouse(&mut self, x: usize, y: usize) {
        let color = TextModeColor::new(Color16::Red, Color16::Red);
        let blch = ScreenCharacter::new(b' ', color);

        self.pointer.x = x;
        self.pointer.y = y;
        self.pointer.preserved = Some(self.mode.read_character(self.pointer.x, self.pointer.y));
        self.mode.write_character(x, y, blch);
    }

    pub fn restore_pointer(&mut self) {
        if let Some(character) = self.pointer.preserved {
            self.mode
                .write_character(self.pointer.x, self.pointer.y, character);
        }
    }
}
