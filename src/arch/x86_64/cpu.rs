use x86_64::instructions::port::Port;

pub fn reboot() -> ! {
    unsafe {
        let mut port = Port::<u8>::new(0x64);
        port.write(0xFE);
    }

    loop {
        x86_64::instructions::hlt();
    }
}

