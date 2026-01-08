#![no_std]
#![no_main]
#![feature(asm)]
//panic implementation

use core::panic::PanicInfo;

//this will be called on kernel panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

//overwriting the os entry point

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop{} // 
}

