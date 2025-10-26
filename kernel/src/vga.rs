// Copyright 2025 Yomi OS Development Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! VGA text mode driver for early boot debugging
//!
//! This module provides a simple VGA text mode driver that can be used
//! before the serial port is initialized. It's particularly useful for
//! debugging boot issues when serial output isn't working.
//!
//! The VGA buffer is located at physical address 0xB8000 and provides
//! an 80x25 character display with color attributes.

#![allow(dead_code)]

use core::fmt;

use spin::Mutex;

/// VGA buffer dimensions
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

/// VGA buffer physical address
const VGA_BUFFER_ADDR: usize = 0xB8000;

/// VGA color codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Color attribute combining foreground and background colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Create a new color code from foreground and background colors
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// VGA character with color attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA text buffer (80x25)
#[repr(transparent)]
struct VgaBuffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],
}

/// VGA writer for outputting text
pub struct VgaWriter {
    column: usize,
    row: usize,
    color_code: ColorCode,
    buffer: &'static mut VgaBuffer,
}

impl VgaWriter {
    /// Create a new VGA writer
    ///
    /// # Safety
    ///
    /// This function creates a mutable reference to VGA memory at 0xB8000.
    /// The caller must ensure this is only called once.
    pub unsafe fn new() -> Self {
        Self {
            column: 0,
            row: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            buffer: &mut *(VGA_BUFFER_ADDR as *mut VgaBuffer),
        }
    }

    /// Set the foreground and background colors
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    /// Write a byte to the VGA buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= VGA_WIDTH {
                    self.new_line();
                }

                let row = self.row;
                let col = self.column;

                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };

                self.column += 1;
            }
        }
    }

    /// Write a string to the VGA buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of printable ASCII range
                _ => self.write_byte(0xfe), // ■ character
            }
        }
    }

    /// Write a string at a specific position
    pub fn write_at(&mut self, s: &str, row: usize, col: usize) {
        if row >= VGA_HEIGHT || col >= VGA_WIDTH {
            return;
        }

        self.row = row;
        self.column = col;
        self.write_string(s);
    }

    /// Move to the next line
    fn new_line(&mut self) {
        if self.row >= VGA_HEIGHT - 1 {
            // Scroll up
            for row in 1..VGA_HEIGHT {
                for col in 0..VGA_WIDTH {
                    self.buffer.chars[row - 1][col] = self.buffer.chars[row][col];
                }
            }
            self.clear_row(VGA_HEIGHT - 1);
        } else {
            self.row += 1;
        }
        self.column = 0;
    }

    /// Clear a row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..VGA_WIDTH {
            self.buffer.chars[row][col] = blank;
        }
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self) {
        for row in 0..VGA_HEIGHT {
            self.clear_row(row);
        }
        self.row = 0;
        self.column = 0;
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global VGA writer
pub static VGA: Mutex<Option<VgaWriter>> = Mutex::new(None);

/// Initialize the VGA writer
///
/// This should be called early in the boot process before serial output
/// is available.
///
/// # Safety
///
/// This function must only be called once during boot.
pub unsafe fn init() {
    let mut vga = VGA.lock();
    *vga = Some(VgaWriter::new());
}

/// Write to VGA (for use in macros)
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    if let Some(ref mut writer) = *VGA.lock() {
        writer.write_fmt(args).ok();
    }
}

/// Print to VGA without newline
#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!($($arg)*))
    };
}

/// Print to VGA with newline
#[macro_export]
macro_rules! vga_println {
    () => ($crate::vga_print!("\n"));
    ($fmt:expr) => ($crate::vga_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::vga_print!(concat!($fmt, "\n"), $($arg)*));
}

/// Write a diagnostic message to the top-right corner of the screen
///
/// This is useful for early boot debugging when you want to indicate
/// that certain stages of boot have been reached.
pub fn write_diagnostic(msg: &str) {
    if let Some(ref mut writer) = *VGA.lock() {
        let col = VGA_WIDTH.saturating_sub(msg.len());
        writer.write_at(msg, 0, col);
    }
}
