#![no_std]
#![no_main]

//! SDI debug print

use core::ptr::{read_volatile, write_volatile};

use ch32_hal as hal;
use core::convert::Infallible;
use hal::debug::SDIPrint;
use hal::delay::Delay;
use hal::gpio::{Input, Level, Output, Pull};
use portable_atomic::{AtomicBool, Ordering};
use ufmt::uWrite;

const DATA0: *mut u32 = 0xE000_00F4 as *mut u32;
const DATA1: *mut u32 = 0xE000_00F8 as *mut u32;
const SPIN_LIMIT: u32 = 100_000; // ~50ms @ 8MHz

static SDI_ALIVE: AtomicBool = AtomicBool::new(true);

pub struct Sdi;

impl uWrite for Sdi {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        sdi_write(s.as_bytes());
        Ok(())
    }
}

macro_rules! sdi_println {
    ($($arg:tt)*) => {
        { let _ = ufmt::uwriteln!(&mut $crate::Sdi, $($arg)*); }
    };
}

fn sdi_write(mut buf: &[u8]) {
    if !SDI_ALIVE.load(Ordering::Relaxed) {
        return;
    }
    while !buf.is_empty() {
        let n = buf.len().min(7);
        let mut pkt = [0u8; 8];
        pkt[0] = n as u8;
        pkt[1..1 + n].copy_from_slice(&buf[..n]);

        let mut spins = 0u32;
        while unsafe { read_volatile(DATA0) } != 0 {
            spins += 1;
            if spins > SPIN_LIMIT {
                SDI_ALIVE.store(false, Ordering::Relaxed);
                return; // nobody listening; give up for good
            }
        }

        let w1 = u32::from_le_bytes(pkt[4..8].try_into().unwrap());
        let w0 = u32::from_le_bytes(pkt[0..4].try_into().unwrap());
        unsafe {
            write_volatile(DATA1, w1); // payload first
            write_volatile(DATA0, w0); // then trigger
        }
        buf = &buf[n..];
    }
}

fn chip_info() {
    // Chip
    let chip_id = hal::signature::chip_id();
    sdi_println!(">> CHIP: {} / DevID: {}", chip_id.name(), chip_id.dev_id());
    // Flash
    sdi_println!(">> FLASH_SIZE: {}kb", hal::signature::flash_size_kb());
    // Unique ID
    sdi_println!(">> CHIP_ID: {:?}", hal::signature::unique_id());
    // Clocks
    let clocks = hal::rcc::clocks();
    sdi_println!(
        ">> CLOCKS: sysclk={} hclk={} pclk1={} pclk2={}",
        clocks.sysclk.0,
        clocks.hclk.0,
        clocks.pclk1.0,
        clocks.pclk2.0
    );
}

#[qingke_rt::entry]
fn main() -> ! {
    SDIPrint::enable();
    unsafe { write_volatile(DATA0, 0) };

    let config = hal::Config::default();
    let p = hal::init(config);
    let mut delay = Delay;

    sdi_println!(">> INIT");
    // Reset
    let rst = hal::pac::RCC.rstsckr().read();
    sdi_println!(">> RST: 0x{:x}", rst.0);

    chip_info();

    // Clear RST and DATA0 (for sdi_write)
    hal::pac::RCC.rstsckr().modify(|w| w.set_rmvf(true)); // clear for next time
    unsafe { write_volatile(DATA0, 0) };

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
    sdi_write(b"!! PANIC !!\n");
    loop {}
}
