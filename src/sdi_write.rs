use core::ptr::{read_volatile, write_volatile};

use portable_atomic::{AtomicBool, Ordering};

const DATA0: *mut u32 = 0xE000_00F4 as *mut u32;
const DATA1: *mut u32 = 0xE000_00F8 as *mut u32;
const SPIN_LIMIT: u32 = 100_000; // ~50ms @ 8MHz

static SDI_ALIVE: AtomicBool = AtomicBool::new(true);

pub fn sdi_clear() {
    unsafe { write_volatile(DATA0, 0) };
}

pub fn sdi_write(mut buf: &[u8]) {
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

#[macro_export]
macro_rules! sdi_writeln {
    ($($arg:expr),*) => {
        $(
            $crate::sdi_write::sdi_write($arg);
            $crate::sdi_write::sdi_write(b" ");
        )*
        $crate::sdi_write::sdi_write(b"\n");
    };
}

// Formatting Traits - in most cases easier to use `ufmt`
// (wont be linked if not used)

const HEX: &[u8; 16] = b"0123456789abcdef";
const DEC: &[u8; 10] = b"0123456789";

pub trait FormatDec {
    type Output: AsRef<[u8]>;
    fn fmt_dec(self) -> Self::Output;
}

pub trait FormatHex {
    type Output: AsRef<[u8]>;
    fn fmt_hex(self) -> Self::Output;
}

impl FormatDec for u32 {
    type Output = [u8; 10];
    fn fmt_dec(self) -> Self::Output {
        let mut buf = [b' '; 10];
        if self == 0 {
            buf[9] = b'0';
            return buf;
        }
        let mut v = self;
        let mut pos = 10;
        while v > 0 {
            pos -= 1;
            buf[pos] = DEC[(v % 10) as usize];
            v /= 10;
        }
        buf
    }
}

impl FormatHex for u64 {
    type Output = [u8; 18];
    fn fmt_hex(self) -> [u8; 18] {
        let bytes = self.to_be_bytes();
        [
            b'0',
            b'x',
            HEX[(bytes[0] >> 4) as usize],
            HEX[(bytes[0] & 0xF) as usize],
            HEX[(bytes[1] >> 4) as usize],
            HEX[(bytes[1] & 0xF) as usize],
            HEX[(bytes[2] >> 4) as usize],
            HEX[(bytes[2] & 0xF) as usize],
            HEX[(bytes[3] >> 4) as usize],
            HEX[(bytes[3] & 0xF) as usize],
            HEX[(bytes[4] >> 4) as usize],
            HEX[(bytes[4] & 0xF) as usize],
            HEX[(bytes[5] >> 4) as usize],
            HEX[(bytes[5] & 0xF) as usize],
            HEX[(bytes[6] >> 4) as usize],
            HEX[(bytes[6] & 0xF) as usize],
            HEX[(bytes[7] >> 4) as usize],
            HEX[(bytes[7] & 0xF) as usize],
        ]
    }
}

impl FormatHex for u32 {
    type Output = [u8; 10];
    fn fmt_hex(self) -> [u8; 10] {
        let bytes = self.to_be_bytes();
        [
            b'0',
            b'x',
            HEX[(bytes[0] >> 4) as usize],
            HEX[(bytes[0] & 0xF) as usize],
            HEX[(bytes[1] >> 4) as usize],
            HEX[(bytes[1] & 0xF) as usize],
            HEX[(bytes[2] >> 4) as usize],
            HEX[(bytes[2] & 0xF) as usize],
            HEX[(bytes[3] >> 4) as usize],
            HEX[(bytes[3] & 0xF) as usize],
        ]
    }
}

impl FormatHex for u16 {
    type Output = [u8; 6];
    fn fmt_hex(self) -> [u8; 6] {
        let bytes = self.to_be_bytes();
        [
            b'0',
            b'x',
            HEX[(bytes[0] >> 4) as usize],
            HEX[(bytes[0] & 0xF) as usize],
            HEX[(bytes[1] >> 4) as usize],
            HEX[(bytes[1] & 0xF) as usize],
        ]
    }
}

impl FormatHex for u8 {
    type Output = [u8; 4];
    fn fmt_hex(self) -> [u8; 4] {
        let bytes = self.to_be_bytes();
        [
            b'0',
            b'x',
            HEX[(bytes[0] >> 4) as usize],
            HEX[(bytes[0] & 0xF) as usize],
        ]
    }
}
