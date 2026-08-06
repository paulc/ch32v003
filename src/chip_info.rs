use ch32_hal as hal;

use crate::sdi_println;

pub fn chip_info() {
    // Chip
    let chip_id = hal::signature::chip_id();
    sdi_println!(
        ">> CHIP [rev_id,dev_id]: 0x{:04x} 0x{:04x}",
        chip_id.rev_id(),
        chip_id.dev_id()
    );
    // Flash
    sdi_println!(">> FLASH_SIZE (KB): {}", hal::signature::flash_size_kb());
    // Unique ID
    let mut buf = [0_u8; 8];
    buf.copy_from_slice(&hal::signature::unique_id()[..8]);
    let id = u64::from_le_bytes(buf);
    sdi_println!(">> CHIP_ID: 0x{:x}", id);
    // Clocks
    let clocks = hal::rcc::clocks();
    sdi_println!(
        ">> CLOCKS [sysclk,hclk,pclk1,pclk2] {} {} {} {}",
        &clocks.sysclk.0,
        &clocks.hclk.0,
        &clocks.pclk1.0,
        &clocks.pclk2.0
    );
}
