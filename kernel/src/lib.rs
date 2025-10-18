#![no_std]

use core::panic::PanicInfo;

/// Kernel initialization function
pub fn init() {
    // Kernel initialization code will go here
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
