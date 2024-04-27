#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use alloc::sync::Arc;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;

use esp_hal::{
    clock::ClockControl,
    embassy, entry,
    i2c::I2C,
    macros::main,
    peripherals::{Peripherals, I2C0},
    prelude::_fugit_RateExtU32,
    system::SystemExt,
    timer::TimerGroup,
    Rng, IO,
};

mod game;
mod snake;
use game::Game;

use ssd1306::{
    mode::BufferedGraphicsMode, prelude::I2CInterface, rotation::DisplayRotation,
    size::DisplaySize128x64, I2CDisplayInterface, Ssd1306,
};
extern crate alloc;
use core::mem::MaybeUninit;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

pub type DisplayController = Arc<
    Mutex<
        CriticalSectionRawMutex,
        Ssd1306<
            I2CInterface<I2C<'static, I2C0>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>,
        >,
    >,
>;

#[main]
async fn main(_spawner: Spawner) -> ! {
    init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    embassy::init(&clocks, timg0);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        400.kHz(),
        &clocks,
    );

    let mut rng = Rng::new(peripherals.RNG);

    let interface = I2CDisplayInterface::new(i2c);

    let display: DisplayController = Arc::new(Mutex::new(
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode(),
    ));

    let up_button = Arc::new(io.pins.gpio14.into_pull_down_input());
    let down_button = Arc::new(io.pins.gpio12.into_pull_down_input());

    let right_button = Arc::new(io.pins.gpio32.into_pull_down_input());
    let left_button = Arc::new(io.pins.gpio33.into_pull_down_input());

    let mut game = Game::new(128, 64, display);

    loop {
        let fastrng = fastrand::Rng::with_seed(rng.random() as u64);

        game.handle_input(
            Arc::clone(&up_button),
            Arc::clone(&down_button),
            Arc::clone(&left_button),
            Arc::clone(&right_button),
        );
        game.update(fastrng);
        game.draw().await;

        Timer::after(Duration::from_millis(100)).await;
    }
}
