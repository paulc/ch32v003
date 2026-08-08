#[cfg(not(feature = "debug"))]
use crate::sdi_write::FormatHex;
#[cfg(not(feature = "debug"))]
use crate::sdi_writeln;
#[cfg(feature = "debug")]
use crate::{sdi_print, sdi_println};

#[cfg(feature = "debug")]
pub fn chip_info() {
    // Chip
    let chip_id = ch32_hal::signature::chip_id();
    sdi_println!(
        ">> CHIP [rev_id,dev_id]: 0x{:04x} 0x{:04x}",
        chip_id.rev_id(),
        chip_id.dev_id()
    );
    // Flash
    sdi_println!(
        ">> FLASH_SIZE (KB): {}",
        ch32_hal::signature::flash_size_kb()
    );
    // Unique ID
    let mut buf = [0_u8; 8];
    buf.copy_from_slice(&ch32_hal::signature::unique_id()[..8]);
    let id = u64::from_le_bytes(buf);
    sdi_println!(">> CHIP_ID: 0x{:x}", id);
    // Clocks
    let clocks = ch32_hal::rcc::clocks();
    sdi_println!(
        ">> CLOCKS [sysclk,hclk,pclk1,pclk2] {} {} {} {}",
        &clocks.sysclk.0,
        &clocks.hclk.0,
        &clocks.pclk1.0,
        &clocks.pclk2.0
    );
}

pub fn decode_reset(rst: u32) {
    #[cfg(feature = "debug")]
    {
        sdi_print!(">> RST: 0x{:x} ", rst);
        if rst & (1 << 26) != 0 {
            sdi_print!(" PIN");
        }
        if rst & (1 << 27) != 0 {
            sdi_print!(" POR");
        }
        if rst & (1 << 28) != 0 {
            sdi_print!(" SFT");
        }
        if rst & (1 << 29) != 0 {
            sdi_print!(" IWDG");
        }
        if rst & (1 << 30) != 0 {
            sdi_print!(" WWDG");
        }
        if rst & (1 << 31) != 0 {
            sdi_print!(" LPWR");
        }
        sdi_print!("\n");
    }
    #[cfg(not(feature = "debug"))]
    {
        sdi_writeln!(b">> RST: ", &rst.fmt_hex());
    }
}

pub fn clear_reset() {
    ch32_hal::pac::RCC.rstsckr().modify(|w| w.set_rmvf(true));
}

#[cfg(feature = "debug")]
pub fn decode_mcause() {
    let mcause: u32;
    let mepc: u32;
    unsafe {
        core::arch::asm!("csrr {}, mcause", out(reg) mcause);
        core::arch::asm!("csrr {}, mepc",   out(reg) mepc);
    }
    sdi_println!(">> MCAUSE: 0x{:x} MEPC: 0x{:x}", mcause, mepc);
}
