use bootloader::boot_info::{FrameBufferInfo, PixelFormat};
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::UnicodeFonts;

// Additional vertical space between lines
const LINE_SPACING: usize = 2;

// BLOCK_SIZE is the number of pixels in a block
pub const BLOCK_SIZE: usize = 8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Black,
    White,
    Grey,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    LightGreen,
    LightRed,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightYellow,
    DarkGreen,
    DarkRed,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    DarkYellow,
}

pub struct Display {
    framebuffer: Option<&'static mut [u8]>,
    pub info: Option<FrameBufferInfo>,
    pub x_pos: usize,
    pub y_pos: usize,
    pub color: Color,
    pub background_color: Color,
}

impl Display {
    pub fn new() -> Self {
        Display{
            framebuffer: None,
            info: None,
            x_pos: 0,
            y_pos: 0,
            color: Color::Green,
            background_color: Color::Black,
        }
    }

    pub fn set_framebuffer(&mut self, framebuffer: &'static mut bootloader::boot_info::FrameBuffer) {
        self.info = Some(framebuffer.info().clone());
        self.framebuffer = Some(framebuffer.buffer_mut());
    }
    
    pub fn set_xy(&mut self, x: usize, y: usize) {
        self.x_pos = x;
        self.y_pos = y;
    }

    fn newline(&mut self) {
        self.y_pos += 8 + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = 0;
    }

    /// Erases all text on the screen.
    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.framebuffer.as_mut().unwrap().fill(0);
    }

    fn width(&self) -> usize {
        self.info.unwrap().horizontal_resolution
    }

    fn height(&self) -> usize {
        self.info.unwrap().vertical_resolution
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                if self.x_pos >= self.width() {
                    self.newline();
                }
                if self.y_pos >= (self.height() - 8) {
                    self.clear();
                }
                let rendered = font8x8::BASIC_FONTS
                    .get(c)
                    .expect("character not found in basic font");
                self.write_rendered_char(rendered);
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: [u8; 8]) {
        for (y, byte) in rendered_char.iter().enumerate() {
            for (x, bit) in (0..8).enumerate() {
                let color = if *byte & (1 << bit) == 0 { self.background_color } else { self.color };
                self.write_pixel(self.x_pos + x, self.y_pos + y, color);
            }
        }
        self.x_pos += 8;
    }

    pub fn write_pixel(&mut self, mut x: usize, mut y: usize, color: Color) {
        x = x % self.info.unwrap().horizontal_resolution;
        y = y % self.info.unwrap().vertical_resolution;
        let pixel_offset = y * self.info.unwrap().stride + x;
        let (r,g,b) = match color {
            Color::Black => (0,0,0),
            Color::Grey => (0x80,0x80,0x80),
            Color::Red => (255,0,0),
            Color::Green => (0,255,0),
            Color::Yellow => (255,255,0),
            Color::Blue => (0,0,255),
            Color::Magenta => (255,0,255),
            Color::Cyan => (0,255,255),
            Color::White => (255,255,255),
            Color::LightGreen => (0,128,0),
            Color::LightRed => (128,0,0),
            Color::LightBlue => (0,0,128),
            Color::LightMagenta => (128,0,128),
            Color::LightCyan => (0,128,128),
            Color::LightYellow => (128,128,0),
            Color::DarkGreen => (0,64,0),
            Color::DarkRed => (64,0,0),
            Color::DarkBlue => (0,0,64),
            Color::DarkMagenta => (64,0,64),
            Color::DarkCyan => (0,64,64),
            Color::DarkYellow => (64,64,0),
        };
        
        let color = match self.info.unwrap().pixel_format {
            PixelFormat::RGB => [r,g,b, 0],
            PixelFormat::BGR => [b,g,r, 0],
            PixelFormat::U8 | _ => [if r+g+b < 255 { 0xf } else { 0 }, 0, 0, 0],
        };
        let bytes_per_pixel = self.info.unwrap().bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer.as_mut().unwrap()[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer.as_mut().unwrap()[byte_offset]) };
    }


    // write_block draws a square block of the specified color at the specified
    pub fn write_block(&mut self, x: usize, y: usize, color: Color) {
        for i in 0..BLOCK_SIZE {
            for j in 0..BLOCK_SIZE {
                self.write_pixel(x + i, y + j, color);
            }
        }
    }
   
    // draw_border draws a border around the screen with a one block padding
    pub fn draw_borders(&mut self) {
        for i in BLOCK_SIZE..self.width()-2*BLOCK_SIZE {
            self.write_block(i, BLOCK_SIZE, Color::DarkGreen);
            self.write_block(i, self.height()-2*BLOCK_SIZE, Color::DarkGreen);
        }
        for i in BLOCK_SIZE..self.height()-2*BLOCK_SIZE {
            self.write_block(BLOCK_SIZE, i, Color::DarkGreen);
            self.write_block(self.width()-2*BLOCK_SIZE, i, Color::DarkGreen);
        }
    }
            
}

unsafe impl Send for Display {}
unsafe impl Sync for Display {}

impl Write for Display {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}