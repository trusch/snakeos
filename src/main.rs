#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use futures_util::stream::StreamExt;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use task::keyboard::ScancodeStream;
use task::tick::TickStream;

mod allocator;
mod display;
mod game2048;
mod gdt;
mod interrupts;
mod memory;
mod serial;
mod snake;
mod task;
mod world;

entry_point!(kernel_main);

use display::{Color, Display};
lazy_static! {
    static ref DISPLAY: spin::Mutex<Display> = spin::Mutex::new(Display::new());
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("init system...");
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    let phys_mem_offset =
        x86_64::VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    serial_println!("init done!");

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        DISPLAY.lock().set_framebuffer(framebuffer);
        DISPLAY.lock().clear();
        // welcome();
        // DISPLAY.lock().clear();
        // DISPLAY.lock().draw_borders();

        use task::executor::Executor;
        use task::Task;

        let mut executor = Executor::new();
        let (width, height) = {
            let display = DISPLAY.lock();
            (
                display.info.unwrap().horizontal_resolution,
                display.info.unwrap().vertical_resolution,
            )
        };
        serial_println!("width: {}, height: {}", width, height);
        let game_snake = Box::new(snake::world::World::new(width, height));
        let game_2048 = Box::new(game2048::World::new(width, height));
        let mut world = Arc::new(spin::Mutex::new(world::World::new(width, height)));
        world.lock().add_game(game_snake, "snake");
        world.lock().add_game(game_2048, "2048");
        serial_println!("enable interrupts");
        x86_64::instructions::interrupts::enable();

        executor.spawn(Task::new(handle_keypresses(Arc::clone(&world))));
        executor.spawn(Task::new(handle_ticks(Arc::clone(&world))));

        serial_println!("start run");
        executor.run();
    }

    serial_println!("no framebuffer!");
    hlt_loop();
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

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

async fn handle_ticks(world: Arc<spin::Mutex<world::World>>) {
    serial_println!("handle_ticks");
    let mut stream = TickStream::new();
    serial_println!("handle_ticks: new()");
    while let Some(_) = stream.next().await {
        // continue;
        // serial_println!("handle_ticks: one tick()");
        let mut world = world.lock();
        // serial_println!("handle_ticks: one tick() done");
        world.on_tick(&mut DISPLAY.lock());
    }
}

async fn handle_keypresses(world: Arc<spin::Mutex<world::World>>) {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // serial_print!("KEY PRESS {:?}\n", key);
                match key {
                    DecodedKey::Unicode(character) => {
                        serial_print!("{}", character);
                    }
                    DecodedKey::RawKey(key) => {
                        serial_print!("{:?}", key);
                    }
                }
                world.lock().on_keypress(key, &mut DISPLAY.lock());
            }
        }
    }
}
