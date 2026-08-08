#![no_std]
#![no_main]

use ch32_hal::bind_interrupts;
use ch32_hal::debug::SDIPrint;
use ch32_hal::gpio::{Input, Level, Output, Pull};
use ch32_hal::i2c::I2c;
use ch32_hal::mode::Async;
use ch32_hal::peripherals::I2C1;
use ch32_hal::time::Hertz;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};

use ch32_util::chip_info::{chip_info, decode_mcause, decode_reset};
use ch32_util::sdi_println;
use ch32_util::ssd1306_async::{Rotation, Size, Ssd1306};

bind_interrupts!(struct Irqs {
    I2C1_EV => ch32_hal::i2c::EventInterruptHandler<ch32_hal::peripherals::I2C1>;
    I2C1_ER => ch32_hal::i2c::ErrorInterruptHandler<ch32_hal::peripherals::I2C1>;
});

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>, interval_ms: u64) {
    loop {
        // sdi_println!(">> led_task: {}", embassy_time::Instant::now().as_ticks());
        led.set_high();
        Timer::after_millis(interval_ms).await;
        led.set_low();
        Timer::after_millis(interval_ms).await;
    }
}

#[embassy_executor::task]
async fn display_task(i2c: I2c<'static, I2C1, Async>, interval: u64) {
    sdi_println!("Init Display");
    let mut display = Ssd1306::new(i2c, 0x3C, Size::S128x64);
    display.init(Rotation::Deg0).await.ok();
    display.clear().await.ok();
    let mut buf = heapless::String::<16>::new();
    let mut count = 0_u32;
    let mut ticker = Ticker::every(Duration::from_millis(interval));
    loop {
        // sdi_println!(">> Display: {}", count);
        ticker.next().await;
        for line in 0..8 {
            buf.clear();
            let _ = ufmt::uwrite!(&mut buf, "LINE: {} <{}>", line, count);
            if display.write_line(line, buf.as_str()).await.is_err() {
                sdi_println!("!! ERROR: write_line");
            }
        }
        count += 1;
    }
}

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    SDIPrint::enable();

    let mut config = ch32_hal::Config::default();
    config.rcc = ch32_hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI;
    let p = ch32_hal::init(config);

    sdi_println!(">> INIT");

    // Chip Info
    decode_reset(ch32_hal::pac::RCC.rstsckr().read().0);
    decode_mcause();
    chip_info();

    // Clear RST
    ch32_hal::pac::RCC.rstsckr().modify(|w| w.set_rmvf(true));

    let led = Output::new(p.PC3, Level::Low, Default::default());
    let _button = Input::new(p.PC0, Pull::Down);

    sdi_println!("Start led_task");
    match led_task(led, 500) {
        Ok(t) => spawner.spawn(t),
        Err(_) => panic!("Error spawning led_task"),
    }

    let scl = p.PC2;
    let sda = p.PC1;

    let mut i2c = I2c::new(
        p.I2C1,
        scl,
        sda,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        Hertz::hz(400_000),
        Default::default(),
    );

    sdi_println!("Scan I2C bus: START");
    for addr in 1..=127 {
        if i2c.blocking_write(addr, &[0]).is_ok() {
            sdi_println!(">> Found I2C device at address: 0x{:02x}", addr);
        }
    }
    sdi_println!("Scan I2C bus: DONE");

    sdi_println!("Start led_task");
    match display_task(i2c,1000) {
        Ok(t) => spawner.spawn(t),
        Err(_) => panic!("Error spawning display_task"),
    }

    loop {
        Timer::after_millis(1000).await;
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    ch32_util::sdi_write::sdi_write(b"!! PANIC !!\n");
    loop {}
}
