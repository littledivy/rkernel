use core::fmt;

use vga::colors::Color16;
use vga::writers::Graphics640x480x16;
use vga::writers::GraphicsWriter;

trait Writer {
    fn inc(&mut self) {}
    fn dec(&mut self) {}
}

struct CommandWriter {
    pub x: usize,
    pub y: usize,
}

static COMMAND_Y: usize = 480 - 16;

pub static FONT: &'static [u8] = include_bytes!("unifont.font");

impl CommandWriter {
    fn init() -> Self {
        Self {
            x: 16,
            y: COMMAND_Y,
        }
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
        if self.x >= 640 {
            self.x = 0;
            self.y += 16;
        }
    }

    fn dec(&mut self) {
        if self.x == 0 {
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
    pub fn new() -> Self {
        let mode = Graphics640x480x16::new();
        mode.set_mode();
        mode.clear_screen(Color16::Black);

        let mut screen = Self {
            mode,
            cmd: CommandWriter::init(),
            stage: StageWriter::init(),
            pointer: Default::default(),
            curr_command: [0u8; 310],
        };
        screen.draw_character(0, COMMAND_Y, '~', Color16::Pink);
        screen
    }

    pub fn write(&mut self, buf: &[u8], fg: Color16) {
        for (offset, ch) in buf.iter().enumerate() {
            if ch == &0u8 {
                continue;
            };
            if self.stage.y >= COMMAND_Y - 16 {
                // ;)
                *self = Screen::new();
            }
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

    pub fn write_byte(&mut self, ch: u8) {
        self.draw_character(self.cmd.x, self.cmd.y, ch as char, Color16::White);
        self.curr_command[self.cmd.x] = ch;
        self.cmd.inc();
    }

    pub fn clear_command(&mut self) {
        while self.cmd.x != 8 {
            self.pop();
        }
        self.pop();
        self.curr_command = [0u8; 310];
    }

    pub fn pop(&mut self) {
        self.cmd.dec();
        for row in 0..16 {
            for col in 0..8 {
                self.mode
                    .set_pixel(self.cmd.x + col, self.cmd.y + row, Color16::Black);
            }
        }
    }

    pub fn set_mouse(&mut self, x: usize, y: usize) {
        let framebuffer = self.mode.get_frame_buffer();

        let offset = x / 8 + y * (640 / 8);
        let pixel_mask = 0x80 >> (x & 0x07);
        vga::vga::VGA
            .lock()
            .graphics_controller_registers
            .set_bit_mask(pixel_mask);

        let pixel = unsafe { framebuffer.add(offset).read_volatile() };
        if pixel != 0x0 {
            return;
        }
        self.pointer.loc.x = x;
        self.pointer.loc.y = y;

        self.pointer.preserved = Some(Color16::Black);
        self.mode.set_pixel(x, y, Color16::LightRed);
    }

    pub fn restore_pointer(&mut self) {
        if let Some(character) = self.pointer.preserved {
            self.mode
                .set_pixel(self.pointer.loc.x, self.pointer.loc.y, character);
        }
    }
}
