//! FNV-1a 64-bit over explicit little-endian bytes. Used to compare runs across builds.

pub struct Fnv64(u64);

impl Fnv64 {
    pub const fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
    pub fn write_u8(&mut self, b: u8) {
        self.0 ^= b as u64;
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }
    pub fn write_u64(&mut self, v: u64) {
        for i in 0..8 {
            self.write_u8((v >> (i * 8)) as u8);
        }
    }
    pub fn write_f64(&mut self, v: f64) {
        self.write_u64(v.to_bits());
    }
    pub fn finish(&self) -> u64 {
        self.0
    }
}

impl Default for Fnv64 {
    fn default() -> Self {
        Self::new()
    }
}
