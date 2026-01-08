#[cfg(feature = "std")]
pub fn do_hash<T: core::hash::Hash>(t: &T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};
    let mut hasher = DefaultHasher::default();
    t.hash(&mut hasher);
    hasher.finish()
}

#[cfg(not(feature = "std"))]
pub fn do_hash<T: core::hash::Hash>(t: &T) -> u64 {
    use core::hash::Hasher;
    // Simple FNV-1a hasher for no_std, for testing purposes only.
    struct FnvHasher(u64);

    impl core::hash::Hasher for FnvHasher {
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes {
                self.0 ^= *byte as u64;
                self.0 = self.0.wrapping_mul(0x100000001b3);
            }
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }

    let mut hasher = FnvHasher(0xcbf29ce484222325);
    t.hash(&mut hasher);
    hasher.finish()
}
