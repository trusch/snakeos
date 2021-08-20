#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::fmt::Write;
use core::panic::PanicInfo;
use futures_util::stream::StreamExt;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use task::keyboard::ScancodeStream;
use task::tick::TickStream;

mod allocator;
mod display;
mod gdt;
mod interrupts;
mod memory;
mod serial;
mod snake;
mod task;

entry_point!(kernel_main);

use snake::world::{World, Direction};
lazy_static! {
    static ref WORLD: spin::Mutex<World> = spin::Mutex::new(World::new(0, 0));
}

use display::{Display, Color};
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
        welcome();
        DISPLAY.lock().clear();
        DISPLAY.lock().draw_borders();

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
        WORLD.lock().reset(width, height);
        serial_println!("enable interrupts");
        x86_64::instructions::interrupts::enable();

        executor.spawn(Task::new(handle_keypresses()));
        executor.spawn(Task::new(handle_ticks()));
        executor.run();
    }

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

async fn handle_ticks() {
    let mut stream = TickStream::new();
    while let Some(_) = stream.next().await {
        let mut world = WORLD.lock();
        world.step();
        world.draw(&mut DISPLAY.lock());
    }
}

async fn handle_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        serial_print!("{}", character);
                        match character {
                            'a' => { let mut w = WORLD.lock(); 
                                if w.direction != Direction::Right {
                                    w.direction = Direction::Left;
                                }
                            },
                            'd' => { let mut w = WORLD.lock(); 
                                if w.direction != Direction::Left {
                                    w.direction = Direction::Right;
                                }
                            },
                            'w' => { let mut w = WORLD.lock(); 
                                if w.direction != Direction::Down {
                                    w.direction = Direction::Up;
                                }
                            },
                            's' => { let mut w = WORLD.lock(); 
                                if w.direction != Direction::Up {
                                    w.direction = Direction::Down;
                                }
                            },
                            'r' => {
                                use x86_64::instructions::interrupts;
                                interrupts::without_interrupts(|| {
                                    let mut w = WORLD.lock();
                                    let mut d = DISPLAY.lock();
                                    let (width, height) = (w.width, w.height);
                                    d.clear();
                                    d.draw_borders();
                                    w.reset(width, height);
                                });
                            },
                            _ => (),
                        }
                    },
                    DecodedKey::RawKey(key) => {
                        serial_print!("{:?}", key);
                        match key {
                            pc_keyboard::KeyCode::ArrowLeft => WORLD.lock().direction = Direction::Left,
                            pc_keyboard::KeyCode::ArrowRight => WORLD.lock().direction = Direction::Right,
                            pc_keyboard::KeyCode::ArrowUp => WORLD.lock().direction = Direction::Up,
                            pc_keyboard::KeyCode::ArrowDown => WORLD.lock().direction = Direction::Down,
                            _ => {},
                        }
                    },
                }
            }
        }
    }
}

fn welcome() {
    let mut display = DISPLAY.lock();
    let (w, h) = (
        display.info.unwrap().horizontal_resolution, 
        display.info.unwrap().vertical_resolution,
    );
    display.clear();
    display.draw_borders();
    let msg = "<=== Welcome to SnakeOS ===>";
    display.set_xy(
        w / 2 - ((msg.len() / 2) * 8),
        h / 2,
    );
    write!(&mut display, "{}", msg);
    let footer = "by trusch";
    display.set_xy(
        w - footer.len()*8 - 3*display::BLOCK_SIZE,
        h - 4*display::BLOCK_SIZE,
    );
    write!(&mut display, "{}", footer);

    for i in 1..10000 {
        display.write_block(0, 0, Color::Black);
    }
}   
