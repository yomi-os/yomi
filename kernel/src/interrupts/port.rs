//! I/O Port access utilities
//!
//! This module provides safe wrappers for x86-64 I/O port instructions
//! (in/out).

use core::marker::PhantomData;

/// A trait for types that can be read from or written to I/O ports
pub trait PortValue: Copy {
    /// Reads a value from the specified port
    ///
    /// # Safety
    ///
    /// Reading from an I/O port can have side effects.
    unsafe fn read_from_port(port: u16) -> Self;

    /// Writes a value to the specified port
    ///
    /// # Safety
    ///
    /// Writing to an I/O port can have side effects.
    unsafe fn write_to_port(port: u16, value: Self);
}

impl PortValue for u8 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u8;
        core::arch::asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
        value
    }

    unsafe fn write_to_port(port: u16, value: Self) {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

impl PortValue for u16 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u16;
        core::arch::asm!(
            "in ax, dx",
            out("ax") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
        value
    }

    unsafe fn write_to_port(port: u16, value: Self) {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

impl PortValue for u32 {
    unsafe fn read_from_port(port: u16) -> Self {
        let value: u32;
        core::arch::asm!(
            "in eax, dx",
            out("eax") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
        value
    }

    unsafe fn write_to_port(port: u16, value: Self) {
        core::arch::asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// A wrapper for I/O port access
///
/// # Type Parameter
///
/// * `T` - The type of value to be read/written (u8, u16, or u32)
pub struct Port<T: PortValue> {
    port: u16,
    _phantom: PhantomData<T>,
}

impl<T: PortValue> Port<T> {
    /// Creates a new Port instance
    ///
    /// # Arguments
    ///
    /// * `port` - The I/O port address
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }

    /// Reads a value from this port
    ///
    /// # Safety
    ///
    /// Reading from an I/O port can have side effects and should only be done
    /// when appropriate for the hardware device.
    pub unsafe fn read(&mut self) -> T {
        T::read_from_port(self.port)
    }

    /// Writes a value to this port
    ///
    /// # Safety
    ///
    /// Writing to an I/O port can have side effects and should only be done
    /// when appropriate for the hardware device.
    pub unsafe fn write(&mut self, value: T) {
        T::write_to_port(self.port, value);
    }
}
