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

fn sdi_uint(v: u32) {
    const DEC: &[u8; 10] = b"0123456789";
    let mut buf = [0u8; 11];
    for i in 0..10 {
        buf[9 - i] = DEC[((v / 10_u32.pow(i as u32)) % 10) as usize];
    }
    buf[10] = b'\n';
    sdi_write(&buf);
}

fn sdi_hex(v: u32) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut buf = [0u8; 9];
    for i in 0..8 {
        buf[7 - i] = HEX[((v >> (i * 4)) & 0xf) as usize];
    }
    buf[8] = b'\n';
    sdi_write(&buf);
}

#[qingke_rt::entry]
fn main() -> ! {
    SDIPrint::enable();
    unsafe { write_volatile(DATA0, 0) };

    let config = hal::Config::default();
    let p = hal::init(config);
    let mut delay = Delay;

    let rst = hal::pac::RCC.rstsckr().read();
    sdi_write(b">> INIT\n");
    sdi_write(b">> RST: ");
    sdi_hex(rst.0);
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
        if n > 20 {
            led.set_low();
            panic!("Bye");
        }
        led.toggle();
        let _tick = hal::pac::SYSTICK.cnt().read();
        sdi_write(b">> TICK: ");
        sdi_uint(n);
        if button.is_high() {
            sdi_write(b">> BUTTON HIGH\n");
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
