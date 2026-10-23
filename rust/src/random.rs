// Port of static/scenes/_shared/js/random.js. UTF-16 matches charCodeAt.
#[derive(Clone)]
pub struct Random(u32);
impl Random {
    pub fn seeded(seed: &str) -> Self {
        let mut h = 0x811c9dc5u32;
        for unit in seed.encode_utf16() {
            h = (h ^ u32::from(unit)).wrapping_mul(0x01000193);
        }
        h ^= h >> 16;
        h = h.wrapping_mul(0x85ebca6b);
        h ^= h >> 13;
        h = h.wrapping_mul(0xc2b2ae35);
        h ^= h >> 16;
        Self(h)
    }
    pub fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let mut t = (self.0 ^ (self.0 >> 15)).wrapping_mul(1 | self.0);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4294967296.0
    }
}
