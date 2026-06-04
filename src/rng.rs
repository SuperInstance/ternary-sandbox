/// A simple seeded PRNG (xoshiro256++) implemented in pure safe Rust.
/// Provides deterministic randomness for reproducible experiments.
#[derive(Debug, Clone)]
pub struct SeededRng {
    state: [u64; 4],
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        // SplitMix64 to initialize state from a single seed
        let mut s = seed;
        let mut state = [0u64; 4];
        for i in 0..4 {
            s = s.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z = z ^ (z >> 31);
            state[i] = z;
        }
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.rotl(
            self.state[0].wrapping_add(self.state[3]), 23)
            .wrapping_add(self.state[0]);

        let t = self.state[1].wrapping_shl(17);

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.rotl(self.state[3], 45);

        result
    }

    /// Returns a float in [0, 1)
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Returns a float in [lo, hi)
    pub fn next_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }

    /// Returns an integer in [lo, hi] (inclusive)
    pub fn next_int(&mut self, lo: i64, hi: i64) -> i64 {
        if lo >= hi { return lo; }
        let range = (hi - lo + 1) as u64;
        lo + (self.next_u64() % range) as i64
    }

    /// Returns true with the given probability
    pub fn next_bool(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    fn rotl(&self, x: u64, k: u32) -> u64 {
        (x << k) | (x >> (64 - k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_rng_deterministic() {
        let mut a = SeededRng::new(42);
        let mut b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SeededRng::new(1);
        let mut b = SeededRng::new(2);
        let mut same = true;
        for _ in 0..10 {
            if a.next_u64() != b.next_u64() { same = false; break; }
        }
        assert!(!same);
    }

    #[test]
    fn next_f64_in_range() {
        let mut rng = SeededRng::new(123);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn next_int_in_range() {
        let mut rng = SeededRng::new(456);
        for _ in 0..1000 {
            let v = rng.next_int(3, 7);
            assert!(v >= 3 && v <= 7);
        }
    }
}
