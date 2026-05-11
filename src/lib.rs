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
    pub fn new(key: [u32; 4]) -> Self {
        Self { key }
    }

    /// Permutes a 96-bit block.
    ///
    /// The block is represented as three `u32` words.
    pub fn permute(&self, block: [u32; 3]) -> [u32; 3] {
        let mut a = block[0];
        let mut b = block[1];
        let mut c = block[2];

        let key_schedule = SpeckKeySchedule::new(self.key);

        for r_key in key_schedule {
            // 1. Speck Round Function on (A, B)
            a = (a.rotate_right(8).wrapping_add(b)) ^ r_key;
            b = b.rotate_left(3) ^ a;

            // 2. Diffuse into C (GFN Logic)
            c ^= b;

            // 3. Word Shuffle: (A, B, C) -> (B, C, A)
            let temp = a;
            a = b;
            b = c;
            c = temp;
        }

        [a, b, c]
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
        if ROUNDS <= self.round {
            return None;
        }

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
}
