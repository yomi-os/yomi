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

//! Serial port driver (UART 16550)
//!
//! This module provides a driver for the 16550 UART serial port,
//! which is used for kernel debugging output via QEMU serial console.

use crate::interrupts::port::Port;
use core::fmt;
use spin::Mutex;

/// Serial port port numbers
const COM1: u16 = 0x3F8;

/// UART register offsets
const DATA: u16 = 0; // Data register (R/W)
const INT_ENABLE: u16 = 1; // Interrupt enable register
const FIFO_CTRL: u16 = 2; // FIFO control register
const LINE_CTRL: u16 = 3; // Line control register
const MODEM_CTRL: u16 = 4; // Modem control register
const LINE_STATUS: u16 = 5; // Line status register

/// Line status flags
const LINE_STATUS_OUTPUT_EMPTY: u8 = 0x20;
const LINE_STATUS_DATA_READY: u8 = 0x01;

/// Serial port
pub struct SerialPort {
    data: Port<u8>,
    int_enable: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_status: Port<u8>,
}

impl SerialPort {
    /// Create new serial port (uninitialized)
    const fn new(base: u16) -> Self {
        Self {
            data: Port::new(base + DATA),
            int_enable: Port::new(base + INT_ENABLE),
            fifo_ctrl: Port::new(base + FIFO_CTRL),
            line_ctrl: Port::new(base + LINE_CTRL),
            modem_ctrl: Port::new(base + MODEM_CTRL),
            line_status: Port::new(base + LINE_STATUS),
        }
    }

    /// Initialize serial port
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.int_enable.write(0x00);

            // Enable baud rate configuration (DLAB = 1)
            self.line_ctrl.write(0x80);

            // Baud rate: 115200 bps (divisor = 1)
            self.data.write(0x01); // Divisor Low
            self.int_enable.write(0x00); // Divisor High

            // 8 bits, no parity, 1 stop bit (DLAB = 0)
            self.line_ctrl.write(0x03);

            // Enable FIFO, 14-byte threshold
            self.fifo_ctrl.write(0xC7);

            // Data Terminal Ready, Request To Send, Output 2
            self.modem_ctrl.write(0x0B);

            // Test: Set to loopback mode
            self.modem_ctrl.write(0x1E);

            // Send test byte
            self.data.write(0xAE);

            // Wait for data to be available
            let mut timeout = 10000;
            while self.line_status.read() & LINE_STATUS_DATA_READY == 0 {
                timeout -= 1;
                if timeout == 0 {
                    // Timeout: Serial port is faulty
                    return;
                }
                core::hint::spin_loop();
            }

            // Check if same byte can be received
            if self.data.read() != 0xAE {
                // Serial port is faulty
                return;
            }

            // Exit loopback mode, return to normal operation
            self.modem_ctrl.write(0x0F);
        }
    }

    /// Send 1 byte
    fn send(&mut self, byte: u8) {
        unsafe {
            // Wait until transmission buffer is empty
            let mut timeout = 100000;
            while self.line_status.read() & LINE_STATUS_OUTPUT_EMPTY == 0 {
                timeout -= 1;
                if timeout == 0 {
                    return; // Hardware failure: transmit buffer never emptied
                }
                core::hint::spin_loop();
            }

            self.data.write(byte);
        }
    }

    /// Send string
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.send(byte);
        }
    }

    /// Receive 1 byte (None if no data)
    pub fn receive(&mut self) -> Option<u8> {
        unsafe {
            if self.line_status.read() & LINE_STATUS_DATA_READY != 0 {
                Some(self.data.read())
            } else {
                None
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        SerialPort::write_str(self, s);
        Ok(())
    }
}

/// Global serial port (COM1)
pub static SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPort::new(COM1));

/// Initialize serial port
pub fn init() {
    SERIAL1.lock().init();
}

/// Print string to serial port
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// Serial output macro
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Serial output macro (with newline)
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
