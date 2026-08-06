#![no_std]
#![no_main]

use ch32_util::sdi_println;
use ch32_util::sdi_write::sdi_clear;

use ch32_hal as hal;
use hal::debug::SDIPrint;
use hal::delay::Delay;
use hal::gpio::{Input, Level, Output, Pull};

#[qingke_rt::entry]
fn main() -> ! {
    SDIPrint::enable();
    sdi_clear();

    let config = hal::Config::default();
    let p = hal::init(config);
    let mut delay = Delay;

    sdi_println!(">> INIT");

    // Reset Status
    let rst = hal::pac::RCC.rstsckr().read();
    sdi_println!(">> RST: {:x}", rst.0);

    // Clear RST and DATA0 (for sdi_write)
    hal::pac::RCC.rstsckr().modify(|w| w.set_rmvf(true)); // clear for next time

    ch32_util::chip_info::chip_info();

    let mut led = Output::new(p.PC3, Level::Low, Default::default());
    let button = Input::new(p.PC0, Pull::Down);

    for _ in 0..10 {
        led.toggle();
        delay.delay_ms(50);
    }

    let mut n = 0_u32;
    loop {
        if n > 100 {
            led.set_low();
            panic!("Bye");
        }

        led.toggle();

        let tick = hal::pac::SYSTICK.cnt().read();
        sdi_println!(">> TICK: {} [{}]", n, tick);

        if button.is_high() {
            sdi_println!(">> BUTTON HIGH");
        }

        delay.delay_ms(250);
        n += 1;
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    ch32_util::sdi_write::sdi_write(b"!! PANIC !!\n");
    loop {}
}
