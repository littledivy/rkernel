use vga::colors::Color16;
use vga::registers::PlaneMask;
use vga::vga::VGA;
use vga::writers::Graphics320x240x256;
use vga::writers::GraphicsWriter;

pub struct Screen {
    mode: Graphics320x240x256,
    x: usize,
    y: usize,
}

impl Screen {
    pub fn new() -> Self {
        let mode = Graphics320x240x256::new();
        mode.set_mode();
        mode.clear_screen(0);
        Self { mode, x: 0, y: 1 }
    }

    fn inc(&mut self) {
        self.x += 8;
        if self.x > 320 {
            self.x = 1;
            self.y += 10;
        }
    }

    fn dec(&mut self) {
        if self.x < 8 {
            self.x = 320;
            if self.y > 10 {
                self.y -= 10;
            }
        } else {
            self.x -= 8;
        }
    }

    pub fn write(&mut self, buf: &[u8]) {
        for (offset, character) in buf.iter().enumerate() {
            self.write_byte(*character);
        }
    }

    pub fn write_byte(&mut self, ch: u8) {
        self.mode.draw_character(self.x, self.y, ch as char, 255);
        self.inc();
    }

    pub fn pop(&mut self) {
        let frame_buffer = self.mode.get_frame_buffer();

        VGA.lock()
            .sequencer_registers
            .set_plane_mask(PlaneMask::ALL_PLANES);

        for i in 0..8 {
            for bit in 0..8 {
                let offset = (320 * (self.y + i) + (self.x + bit)) / 4;
                unsafe {
                    frame_buffer.add(offset).write_volatile(0);
                }
            }
        }

        self.dec();
    }
}
