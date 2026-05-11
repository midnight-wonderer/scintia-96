#![no_std]

//! # Scintia-96
//!
//! A keyed 96-bit permutation for Rust.
//! It uses a Speck-based round function with a 128-bit key.

/// The number of rounds used in the permutation.
const ROUNDS: u32 = 32;

/// A Scintia-96 instance for performing keyed permutations.
#[derive(Debug, Clone, Copy)]
pub struct Scintia96 {
    key: [u32; 4],
}

impl Scintia96 {
    /// Creates a new Scintia-96 instance from a 128-bit key (4 x u32).
    #[inline]
    pub const fn new(key: [u32; 4]) -> Self {
        Self { key }
    }

    /// Permutes a 96-bit block.
    ///
    /// The block is represented as three `u32` words.
    pub fn permute(&self, block: [u32; 3]) -> [u32; 3] {
        let mut a = block[0];
        let mut b = block[1];
        let mut c = block[2];

        let mut key_schedule = SpeckKeySchedule::new(self.key);

        // Unroll by 3 rounds per iteration
        for _ in 0..(ROUNDS / 3) {
            Self::round_step(&mut key_schedule, &mut a, &mut b, &mut c);
            Self::round_step(&mut key_schedule, &mut b, &mut c, &mut a);
            Self::round_step(&mut key_schedule, &mut c, &mut a, &mut b);
        }

        match ROUNDS % 3 {
            1 => {
                Self::round_step(&mut key_schedule, &mut a, &mut b, &mut c);
                [b, c, a]
            }
            2 => {
                Self::round_step(&mut key_schedule, &mut a, &mut b, &mut c);
                Self::round_step(&mut key_schedule, &mut b, &mut c, &mut a);
                [c, a, b]
            }
            _ => [a, b, c],
        }
    }

    #[inline(always)]
    fn round_step(key_schedule: &mut SpeckKeySchedule, x: &mut u32, y: &mut u32, z: &mut u32) {
        let k = key_schedule.next().unwrap();
        *x = (x.rotate_right(8).wrapping_add(*y)) ^ k;
        *y = y.rotate_left(3) ^ *x;
        *z ^= *y;
    }
}

/// Private iterator for generating round keys on demand.
struct SpeckKeySchedule {
    k: u32,
    l: [u32; 3],
    round: u32,
}

impl SpeckKeySchedule {
    #[inline]
    fn new(key: [u32; 4]) -> Self {
        Self {
            k: key[0],
            l: [key[1], key[2], key[3]],
            round: 0,
        }
    }
}

impl Iterator for SpeckKeySchedule {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current_key = self.k;

        // Key Schedule Update (Speck standard logic)
        // alpha = 8, beta = 3
        let i = self.round;
        let l_idx = (i as usize) % 3;

        let new_l = (self.k.wrapping_add(self.l[l_idx].rotate_right(8))) ^ i;
        let new_k = self.k.rotate_left(3) ^ new_l;

        self.l[l_idx] = new_l;
        self.k = new_k;
        self.round += 1;

        Some(current_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_consistency() {
        let key = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
        let cipher = Scintia96::new(key);

        let block = [0u32, 0u32, 0u32];
        let result1 = cipher.permute(block);
        let result2 = cipher.permute(block);

        assert_eq!(result1, result2);
        assert_ne!(result1, block);
    }

    #[test]
    fn test_const_initialization() {
        const KEY: [u32; 4] = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
        const CIPHER: Scintia96 = Scintia96::new(KEY);
        assert_eq!(CIPHER.key[0], 0x01020304);
    }

    #[test]
    fn test_vectors() {
        let cases = [
            (
                [0, 0, 0, 0],
                [0, 0, 0],
                [0x20e47313, 0x3bd86576, 0x5ed2de89],
            ),
            (
                [0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff],
                [0xffffffff, 0xffffffff, 0xffffffff],
                [0x25fca5a8, 0xe471bef3, 0x7053daa6],
            ),
            (
                [0x12345678, 0x9abcdef0, 0x0fedcba9, 0x87654321],
                [0x11223344, 0x55667788, 0x99aabbcc],
                [0xbf846ba5, 0xe56df4de, 0x0e19b936],
            ),
            (
                [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10],
                [0xdeadbeef, 0xcafebabe, 0x12345678],
                [0x52cbbcb8, 0xfa885f9d, 0x5441aac1],
            ),
        ];

        for (key, input, expected) in cases {
            let cipher = Scintia96::new(key);
            assert_eq!(cipher.permute(input), expected, "Failed for key {:08x?}", key);
        }
    }
}
