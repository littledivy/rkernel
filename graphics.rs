/// `rkernel::graphics::Screen` provides the high level API to draw on the 640x480 screen with 16 layers.
use vga::colors::Color16;
use vga::registers::PlaneMask;
use vga::vga::VGA;
use vga::writers::Graphics320x240x256;
use vga::writers::GraphicsWriter;

trait Writer {
    fn inc(&mut self) {}
    fn dec(&mut self) {}

    // TODO(littledivy): Writers should actually implement their own writing mechanism.
    // Maybe pass VGA mode as mutable?
    // fn write(&mut self, buf: &[u8]) {}
    fn reset(&mut self) {}
}

struct CommandWriter {
    /// Only dynamic value of the command input.
    pub x: usize,
    /// Y remains constant at `WBOUNDARY - 10`.
    /// We refer this when drawing to screen.
    pub y: usize,
}

static CommandY: usize = 230 - 10;

impl CommandWriter {
  fn init() -> Self {
        Self { x: 0, y: CommandY }
  }
}

impl Writer for CommandWriter {
    fn inc(&mut self) {
        self.x += 8;
        if self.x > 310 {
            self.x = 12;
        }
    }

    fn dec(&mut self) {
        if self.x < 8 {
            self.x = 12;
        } else {
            self.x -= 8;
        }
    }
    
    fn reset(&mut self) {
      self.y = CommandY;
      self.x = 0;
    }
}

struct StageWriter {
  x: usize,
  y: usize,
}

impl StageWriter {
  pub fn init() -> Self {
    Self { x: 0, y: 18 }
  }
}

impl Writer for StageWriter {
  fn inc(&mut self) {
    self.x += 8;
    if self.x > 310 {
       self.x = 12;
       self.y += 8;
    }
  }
  
  fn dec(&mut self) {
    if self.x < 8 {
       self.x = 12;
       if self.y < 8 {
         self.y = 18;
       } else {
         self.y -= 8;
       }
    } else {
       self.x -= 8;
    }
  }
  
  fn reset(&mut self) {
    self.y = 18;
    self.x = 0;
  }
}

/// Uh, so this is the screen. Not literally 
pub struct Screen {
    pub mode: Graphics320x240x256,
    pub cmd: CommandWriter,
    stage: StageWriter,
}

impl Screen {
    /// Creates a new screen on top of the 640x480x16 VGA Graphics writer.
    /// - FIles
    pub fn new() -> Self {
        let mode = Graphics320x240x256::new();
        mode.set_mode();
        mode.clear_screen(0);
        // Draws the stage and command input boundaries.
        // Note: Boundaries should be respected on later parts of code.
        mode.draw_line((10, 10), (310, 10), 255);
        mode.draw_line((10, 10), (10, 230), 255);
        mode.draw_line((10, 230), (310, 230), 255);
        mode.draw_line((310, 230), (310, 10), 255);
        mode.draw_line((10, 230 - 12), (310, 230 - 12), 255);
        Self {
            mode,
            cmd: CommandWriter::init(),
            stage: StageWriter::init(),
        }
    }

    /// Writes to the stage.
    pub fn write(&mut self, buf: &[u8]) {
        for (offset, ch) in buf.iter().enumerate() {
            // self.write_byte(*ch);
            self.mode.draw_character(self.stage.x + 10 + 2, self.stage.y, *ch as char, 255);
            self.stage.inc();
        }
    }

    /// Writes to the command input.
    pub fn write_byte(&mut self, ch: u8) {
        self.mode
            .draw_character(self.cmd.x + 10 + 2, self.cmd.y, ch as char, 255);
        self.cmd.inc();
    }
  
    /// Resets the stage.
    pub fn clear_stage(&mut self) {
        
        self.stage.reset();
    }
    
    /// Resets the previous 8x8 character from R to L of the command input.
    pub fn pop(&mut self) {
        let frame_buffer = self.mode.get_frame_buffer();

        VGA.lock()
            .sequencer_registers
            .set_plane_mask(PlaneMask::ALL_PLANES);

        for i in 0..8 {
            for bit in 0..8 {
                let offset = (320 * (self.cmd.y + i) + (self.cmd.x + 10 + 2 + bit)) / 4;
                unsafe {
                    frame_buffer.add(offset).write_volatile(0);
                }
            }
        }

        self.cmd.dec();
    }
}
