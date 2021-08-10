#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{entry_point, BootInfo};
use core::fmt::Write;
use core::panic::PanicInfo;

mod display;
mod gdt;
mod interrupts;
mod serial;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("init system...");
    init();
    serial_println!("init done!");
    
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut display = display::Display::new(framebuffer);
        display.clear();
        let msg = "<=== Welcome to SnakeOS ===>";
        display.set_xy(
            display.info.horizontal_resolution / 2 - ((msg.len() / 2) * 8),
            display.info.vertical_resolution / 2,
        );
        write!(&mut display, "{}", msg);
        let footer = "by trusch";
        display.set_xy(
            display.info.horizontal_resolution - footer.len()*8 - 10, 
            display.info.vertical_resolution - 18,
        );
        write!(&mut display, "{}", footer);
    }
    hlt_loop();
}

fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    hlt_loop();
}
