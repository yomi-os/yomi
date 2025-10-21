//! 8259 PIC (Programmable Interrupt Controller) implementation
//!
//! The 8259 PIC is a legacy interrupt controller used in x86 systems.
//! This module provides initialization and management of the Master/Slave PIC pair.

use super::port::Port;
use spin::Mutex;

/// PIC port numbers
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// PIC initialization commands
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

/// EOI (End of Interrupt) command
const EOI: u8 = 0x20;

/// 8259 PIC (Programmable Interrupt Controller)
struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    /// Creates a new PIC with the given offset and port addresses
    const fn new(offset: u8, command_port: u16, data_port: u16) -> Self {
        Self {
            offset,
            command: Port::new(command_port),
            data: Port::new(data_port),
        }
    }

    /// Sends EOI (End of Interrupt) signal to this PIC
    ///
    /// # Safety
    ///
    /// Must be called from interrupt context after handling an IRQ.
    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(EOI);
    }

    /// Sets the interrupt mask for this PIC
    ///
    /// # Safety
    ///
    /// Writing to PIC ports can affect interrupt handling.
    unsafe fn set_mask(&mut self, mask: u8) {
        self.data.write(mask);
    }

    /// Reads the current interrupt mask
    ///
    /// # Safety
    ///
    /// Reading from PIC ports.
    unsafe fn read_mask(&mut self) -> u8 {
        self.data.read()
    }
}

/// ChainedPics (Master + Slave PIC pair)
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    /// Creates a new ChainedPics with the given vector offsets
    ///
    /// # Arguments
    ///
    /// * `offset1` - Vector offset for master PIC (typically 32)
    /// * `offset2` - Vector offset for slave PIC (typically 40)
    ///
    /// # Safety
    ///
    /// This function is const and doesn't perform any initialization.
    /// The actual hardware initialization happens in `initialize()`.
    pub const unsafe fn new(offset1: u8, offset2: u8) -> Self {
        Self {
            pics: [
                Pic::new(offset1, PIC1_COMMAND, PIC1_DATA),
                Pic::new(offset2, PIC2_COMMAND, PIC2_DATA),
            ],
        }
    }

    /// Initializes the PIC pair
    ///
    /// This function performs the initialization sequence for both PICs,
    /// setting up their vector offsets and cascade configuration.
    ///
    /// # Safety
    ///
    /// This function writes to PIC I/O ports and should only be called once
    /// during kernel initialization.
    pub unsafe fn initialize(&mut self) {
        // Save current masks
        let mask1 = self.pics[0].read_mask();
        let mask2 = self.pics[1].read_mask();

        // Start initialization sequence
        self.pics[0].command.write(ICW1_INIT | ICW1_ICW4);
        io_wait();
        self.pics[1].command.write(ICW1_INIT | ICW1_ICW4);
        io_wait();

        // Set vector offsets
        self.pics[0].data.write(self.pics[0].offset);
        io_wait();
        self.pics[1].data.write(self.pics[1].offset);
        io_wait();

        // Configure cascade mode
        self.pics[0].data.write(4); // Slave PIC connected to IRQ2
        io_wait();
        self.pics[1].data.write(2); // Slave PIC cascade identity
        io_wait();

        // Set 8086 mode
        self.pics[0].data.write(ICW4_8086);
        io_wait();
        self.pics[1].data.write(ICW4_8086);
        io_wait();

        // Restore masks
        self.pics[0].set_mask(mask1);
        self.pics[1].set_mask(mask2);
    }

    /// Sends EOI (End of Interrupt) to the appropriate PIC(s)
    ///
    /// # Arguments
    ///
    /// * `irq` - IRQ number (0-15)
    ///
    /// # Safety
    ///
    /// Must be called from interrupt context after handling an IRQ.
    pub unsafe fn notify_end_of_interrupt(&mut self, irq: u8) {
        // If IRQ came from slave PIC, send EOI to both PICs
        if irq >= 8 {
            self.pics[1].end_of_interrupt();
        }
        // Always send EOI to master PIC
        self.pics[0].end_of_interrupt();
    }

    /// Enables (unmasks) a specific IRQ line
    ///
    /// # Arguments
    ///
    /// * `irq` - IRQ number (0-15)
    ///
    /// # Safety
    ///
    /// Modifies interrupt mask registers.
    pub unsafe fn unmask(&mut self, irq: u8) {
        debug_assert!(irq < 16);
        if irq >= 8 {
            // Ensure master's cascade line (IRQ2) is unmasked
            let m = self.pics[0].read_mask() & !(1u8 << 2);
            self.pics[0].set_mask(m);
        }
        let (pic, line) = if irq < 8 {
            (&mut self.pics[0], irq)
        } else {
            (&mut self.pics[1], irq - 8)
        };
        let mask = pic.read_mask() & !(1u8 << line);
        pic.set_mask(mask);
    }

    /// Disables (masks) a specific IRQ line
    ///
    /// # Arguments
    ///
    /// * `irq` - IRQ number (0-15)
    ///
    /// # Safety
    ///
    /// Modifies interrupt mask registers.
    pub unsafe fn mask(&mut self, irq: u8) {
        let pic = if irq < 8 {
            &mut self.pics[0]
        } else {
            &mut self.pics[1]
        };
        let line = irq % 8;
        let mask = pic.read_mask() | (1 << line);
        pic.set_mask(mask);
    }
}

/// I/O wait function for compatibility with old PICs
///
/// Some old PICs require a delay between successive I/O operations.
/// Writing to port 0x80 provides a short delay.
///
/// # Safety
///
/// Writes to an unused port for timing purposes.
unsafe fn io_wait() {
    Port::<u8>::new(0x80).write(0);
}

/// Global PICS instance
///
/// The PICs are initialized with:
/// - Master PIC offset: 32 (IRQ 0-7 → interrupts 32-39)
/// - Slave PIC offset: 40 (IRQ 8-15 → interrupts 40-47)
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(32, 40) });
