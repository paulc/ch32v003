#![no_std]
#![no_main]

use ch32_hal as hal;
use hal::debug::SDIPrint;
use hal::delay::Delay;
use hal::gpio::{Input, Level, Output, Pull};
use hal::i2c::I2c;
use hal::time::Hertz;

use ch32_util::chip_info::{chip_info, decode_mcause, decode_reset};
use ch32_util::sdi_println;

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[qingke_rt::entry]
fn main() -> ! {
    SDIPrint::enable();

    let config = hal::Config::default();
    let p = hal::init(config);
    let mut delay = Delay;

    sdi_println!(">> INIT");

    // Chip Info
    decode_reset(hal::pac::RCC.rstsckr().read().0);
    decode_mcause();
    chip_info();

    // Clear RST
    hal::pac::RCC.rstsckr().modify(|w| w.set_rmvf(true));

    let mut led = Output::new(p.PC3, Level::Low, Default::default());
    let _button = Input::new(p.PC0, Pull::Down);

    let scl = p.PC2;
    let sda = p.PC1;

    let mut i2c = I2c::new_blocking(p.I2C1, scl, sda, Hertz::hz(400_000), Default::default());

    sdi_println!("Scan I2C bus: START");
    for addr in 1..=127 {
        if i2c.blocking_write(addr, &[0]).is_ok() {
            sdi_println!(">> Found I2C device at address: 0x{:02x}", addr);
        }
    }
    sdi_println!("Scan I2C bus: DONE");

    sdi_println!("Init Display");
    let interface = I2CDisplayInterface::new(i2c);
    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();
    sdi_println!("Init Display: DONE");

    /*
        // TEST DISPLAY
        use ssd1306::command::Command;
        let mut interface = display.release();
        Command::AllOn(true).send(&mut interface).unwrap();
        delay.delay_ms(2000);
        Command::AllOn(false).send(&mut interface).unwrap();
        let mut display =
            Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();
    */

    let mut display_buf = [const { heapless::String::<16>::new() }; 8];
    let mut count = 0_u32;
    loop {
        let _ = display.clear();
        for (row, mut buf) in display_buf.iter_mut().enumerate() {
            let _ = display.set_position(0, row as u8);
            buf.clear();
            let _ = ufmt::uwrite!(&mut buf, "ROW: {} <{}>", row, count);
            for c in buf.as_str().chars() {
                let _ = display.print_char(c);
            }
        }
        led.toggle();
        delay.delay_ms(200);
        count += 1;
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    ch32_util::sdi_write::sdi_write(b"!! PANIC !!\n");
    loop {}
}
