//! The `entropy` CSR's source when seeded: the same stream as the
//! Python core's `random.Random(seed).getrandbits(48)`, so both cores
//! read the same values and can be compared step by step.
//!
//! That is MT19937 seeded with `init_by_array([seed])`, as CPython seeds
//! from a small non-negative integer, and 48 bits built from two 32-bit
//! outputs, the first as the low word.

const N: usize = 624;
const M: usize = 397;

pub struct Entropy {
    state: [u32; N],
    index: usize,
}

impl Entropy {
    pub fn new(seed: u32) -> Self {
        let mut mt = Entropy { state: [0; N], index: N };
        mt.init(19_650_218);
        mt.mix_key(&[seed]);
        mt
    }

    fn init(&mut self, s: u32) {
        self.state[0] = s;
        for i in 1..N {
            let prev = self.state[i - 1];
            self.state[i] =
                1_812_433_253u32.wrapping_mul(prev ^ (prev >> 30)).wrapping_add(i as u32);
        }
    }

    fn mix_key(&mut self, key: &[u32]) {
        let mt = &mut self.state;
        let (mut i, mut j) = (1, 0);
        for _ in 0..N.max(key.len()) {
            let prev = mt[i - 1];
            mt[i] = (mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            let prev = mt[i - 1];
            mt[i] =
                (mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_566_083_941)).wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
    }

    fn twist(&mut self) {
        let mt = &mut self.state;
        for i in 0..N {
            let y = (mt[i] & 0x8000_0000) | (mt[(i + 1) % N] & 0x7fff_ffff);
            let mag = if y & 1 == 1 { 0x9908_b0df } else { 0 };
            mt[i] = mt[(i + M) % N] ^ (y >> 1) ^ mag;
        }
        self.index = 0;
    }

    fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// 48 random bits, as `getrandbits(48)`.
    pub fn next48(&mut self) -> i64 {
        let lo = self.next_u32() as i64;
        let hi = (self.next_u32() >> 16) as i64;
        lo | (hi << 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_python_random() {
        // random.Random(1).getrandbits(32) twice, then getrandbits(48).
        let mut e = Entropy::new(1);
        assert_eq!((e.next_u32(), e.next_u32()), (577_090_037, 2_444_712_010));
        let mut e = Entropy::new(1);
        let got: Vec<i64> = (0..3).map(|_| e.next48()).collect();
        assert_eq!(got, vec![160_215_742_132_725, 225_820_135_221_983, 17_762_970_155_972]);
    }
}
