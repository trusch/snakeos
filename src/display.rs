use bootloader::boot_info::{FrameBufferInfo, PixelFormat};
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::UnicodeFonts;

// Additional vertical space between lines
const LINE_SPACING: usize = 2;

pub struct Display {
    framebuffer: &'static mut [u8],
    pub info: FrameBufferInfo,
    pub x_pos: usize,
    pub y_pos: usize,
}

impl Display {
    pub fn new(framebuffer: &'static mut bootloader::boot_info::FrameBuffer) -> Self {
        Display{
            info: framebuffer.info().clone(),
            framebuffer: framebuffer.buffer_mut(),
            x_pos: 0,
            y_pos: 0,
        }
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
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.horizontal_resolution
    }

    fn height(&self) -> usize {
        self.info.vertical_resolution
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
                let alpha = if *byte & (1 << bit) == 0 { 0 } else { 255 };
                self.write_pixel(self.x_pos + x, self.y_pos + y, alpha);
            }
        }
        self.x_pos += 8;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::RGB => [intensity / 2, intensity, intensity / 2, 0],
            PixelFormat::BGR => [intensity / 2, intensity, intensity / 2, 0],
            PixelFormat::U8 | _ => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
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