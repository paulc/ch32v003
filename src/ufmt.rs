use core::convert::Infallible;
use ufmt::uWrite;

pub struct Sdi;

impl uWrite for Sdi {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        crate::sdi_write::sdi_write(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! sdi_println {
    ($($arg:tt)*) => {
        { let _ = ufmt::uwriteln!(&mut $crate::ufmt::Sdi, $($arg)*); }
    };
}

#[macro_export]
macro_rules! sdi_print {
    ($($arg:tt)*) => {
        { let _ = ufmt::uwrite!(&mut $crate::ufmt::Sdi, $($arg)*); }
    };
}
