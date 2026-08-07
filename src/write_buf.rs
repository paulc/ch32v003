use ufmt::uWrite;

pub struct WriteBuf<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> WriteBuf<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            pos: 0,
        }
    }
    pub fn clear(&mut self) {
        self.pos = 0;
        self.buf.iter_mut().for_each(|b| *b = 0);
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
    pub fn as_str(&self) -> &str {
        // only whole &str chunks are ever written, so this is valid UTF-8
        core::str::from_utf8(&self.buf[..self.pos]).unwrap()
    }
}

impl<const N: usize> uWrite for WriteBuf<N> {
    type Error = (); // "buffer full"

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let bytes = s.as_bytes();
        let end = self.pos.checked_add(bytes.len()).ok_or(())?;
        if end > N {
            return Err(());
        }
        self.buf[self.pos..end].copy_from_slice(bytes);
        self.pos = end;
        Ok(())
    }
}
