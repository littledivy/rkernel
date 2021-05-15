/// `rkernel::graphics::Screen` provides the high level API to draw on the 640x480 screen with 16 layers.
use core::fmt;

use vga::colors::Color16;
use vga::colors::TextModeColor;
use vga::registers::PlaneMask;
use vga::vga::VGA;
use vga::writers::{Graphics640x480x16, GraphicsWriter};
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

static CommandY: usize = 480 - 16;

pub static FONT: &'static [u8] = include_bytes!("unifont.font");

impl CommandWriter {
    fn init() -> Self {
        Self { x: 8, y: CommandY }
    }
}

impl Writer for CommandWriter {
    fn inc(&mut self) {
        self.x += 8;
        if self.x > 640 {
            self.x = 8;
        }
    }

    fn dec(&mut self) {
        if self.x <= 8 {
            self.x = 8;
        } else {
            self.x -= 8;
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
        self.x += 8;
        if self.x > 640 {
            self.x = 0;
            self.y += 16;
        }
    }

    fn dec(&mut self) {
        if self.x < 0 {
            self.x = 0;
            if self.y < 1 {
                self.y = 1;
            } else {
                self.y -= 16;
            }
        } else {
            self.x -= 16;
        }
    }
}

struct Point {
    x: usize,
    y: usize,
}

struct MousePointer {
    loc: Point,
    preserved: Option<Color16>,
}

impl Default for MousePointer {
    fn default() -> Self {
        // Default coordinates must match with `crate::mouse::State` defaults.
        Self {
            loc: Point { x: 0, y: 0 },
            preserved: None,
        }
    }
}

pub struct Screen {
    pub mode: Graphics640x480x16,
    cmd: CommandWriter,
    pub curr_command: [u8; 310],
    stage: StageWriter,
    pointer: MousePointer,
}

impl Screen {
    /// Creates a new screen on top of the 640x480x16 VGA Graphics writer.
    pub fn new() -> Self {
        let mode = Graphics640x480x16::new();
        mode.set_mode();
        mode.clear_screen(Color16::Black);

        let mut screen = Self {
            mode,
            cmd: CommandWriter::init(),
            stage: StageWriter::init(),
            pointer: Default::default(),
            // Allocate memory for command storage
            curr_command: [0u8; 310],
        };
        screen
    }

    /// Writes to the stage.
    pub fn write(&mut self, buf: &[u8], fg: Color16) {
        for (offset, ch) in buf.iter().enumerate() {
            if ch == &0u8 {
                continue;
            };
            if ch == &b'\n' {
                self.stage.x = 0;
                self.stage.y += 16;
            } else {
                self.draw_character(self.stage.x, self.stage.y, *ch as char, fg);
                self.stage.inc();
            }
        }
    }

    pub fn draw_character(&mut self, x: usize, y: usize, ch: char, color: Color16) {
        let font_i = 16 * (ch as usize);
        if font_i + 16 <= FONT.len() {
            for row in 0..16 {
                let row_data = FONT[font_i + row];
                for col in 0..8 {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        self.mode.set_pixel(x + col, y + row, color);
                    }
                }
            }
        }
    }

    /// Writes to the command input.
    pub fn write_byte(&mut self, ch: u8) {
        self.draw_character(self.cmd.x, self.cmd.y, ch as char, Color16::White);
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
        self.mode
            .draw_character(self.cmd.x, self.cmd.y, ' ', Color16::Black);
        self.cmd.dec();
    }

    pub fn set_mouse(&mut self, x: usize, y: usize) {
        self.pointer.loc.x = x;
        self.pointer.loc.y = y;

        self.mode.set_pixel(x, y, Color16::LightRed);
    }

    pub fn restore_pointer(&mut self) {
        if let Some(character) = self.pointer.preserved {
            self.mode
                .set_pixel(self.pointer.loc.x, self.pointer.loc.y, character);
        }
    }
}
