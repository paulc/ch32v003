#![no_std]
#![no_main]

//! SDI debug print

use core::ptr::{read_volatile, write_volatile};

use ch32_hal as hal;
use hal::debug::SDIPrint;
use hal::delay::Delay;
use hal::gpio::{Input, Level, Output, Pull};
use portable_atomic::{AtomicBool, Ordering};

const DATA0: *mut u32 = 0xE000_00F4 as *mut u32;
const DATA1: *mut u32 = 0xE000_00F8 as *mut u32;
const SPIN_LIMIT: u32 = 100_000; // ~50ms @ 8MHz

static SDI_ALIVE: AtomicBool = AtomicBool::new(true);

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

const HEX: &[u8; 16] = b"0123456789abcdef";
const DEC: &[u8; 10] = b"0123456789";

fn sdi_byte(v: u8) {
    let mut buf = [0u8; 2];
    for i in 0..2 {
        buf[1 - i] = HEX[((v >> (i * 4)) & 0xf) as usize];
    }
    sdi_write(&buf);
}

fn sdi_uint(v: u32) {
    let mut buf = [0u8; 10];
    for i in 0..10 {
        buf[9 - i] = DEC[((v / 10_u32.pow(i as u32)) % 10) as usize];
    }
    sdi_write(&buf);
}

fn sdi_hex(v: u32) {
    let mut buf = [0u8; 8];
    for i in 0..8 {
        buf[7 - i] = HEX[((v >> (i * 4)) & 0xf) as usize];
    }
    sdi_write(&buf);
}

fn sdi_nl() {
    sdi_write(b"\n");
}

fn chip_info() {
    // Chip
    let chip_id = hal::signature::chip_id();
    sdi_write(b">> CHIP: ");
    sdi_write(chip_id.name().as_bytes());
    sdi_write(b" DevID: ");
    sdi_hex(chip_id.dev_id() as u32);
    sdi_nl();
    // Flash
    sdi_write(b">> FLASH_SIZE: ");
    sdi_uint(hal::signature::flash_size_kb() as u32);
    sdi_write(b"kb");
    sdi_nl();
    // Unique ID
    let chip_id = hal::signature::unique_id();
    sdi_write(b">> CHIP_ID: ");
    for i in 0..12 {
        sdi_byte(chip_id[i]);
        if i < 11 {
            sdi_write(b":");
        }
    }
    sdi_nl();
    // Clocks
    let clocks = hal::rcc::clocks();
    sdi_write(b">> CLOCKS: sysclk=");
    sdi_uint(clocks.sysclk.0);
    sdi_write(b" hclk=");
    sdi_uint(clocks.hclk.0);
    sdi_write(b" pclk1=");
    sdi_uint(clocks.pclk1.0);
    sdi_write(b" pclk2=");
    sdi_uint(clocks.pclk2.0);
    sdi_nl();
}

#[qingke_rt::entry]
fn main() -> ! {
    SDIPrint::enable();
    unsafe { write_volatile(DATA0, 0) };

    let config = hal::Config::default();
    let p = hal::init(config);
    let mut delay = Delay;

    sdi_write(b">> INIT\n");
    // Reset
    let rst = hal::pac::RCC.rstsckr().read();
    sdi_write(b">> RST: ");
    sdi_hex(rst.0);
    sdi_nl();

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

        sdi_write(b">> COUNT: ");
        sdi_uint(n);
        let tick = hal::pac::SYSTICK.cnt().read();
        sdi_write(b" / SYSTICK: ");
        sdi_uint(tick);
        sdi_nl();

        if button.is_high() {
            sdi_write(b">> BUTTON HIGH\n");
            sdi_nl();
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
