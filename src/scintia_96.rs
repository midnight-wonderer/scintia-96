use crate::ROUNDS;
use crate::key_schedule::{SpeckKeySchedule, encrypt_step};

/// A Scintia-96 instance for performing keyed permutations.
#[derive(Debug, Clone, Copy)]
pub struct Scintia96 {
    key: [u32; 4],
}

impl Scintia96 {
    /// Creates a new Scintia-96 instance from a 128-bit key (4 x u32).
    ///
    /// The key is used to derive round keys on-the-fly during each permutation.
    /// This is the most memory-efficient way to use Scintia-96, ideal for microcontrollers.
    #[inline]
    pub const fn new(key: [u32; 4]) -> Self {
        Self { key }
    }

    /// Permutes a 96-bit block using the provided key.
    ///
    /// The block is represented as three 32-bit words. Since Scintia-96 is a
    /// true permutation, the mapping is one-to-one and guaranteed to be collision-free.
    ///
    /// # Example
    /// ```
    /// # use scintia_96::Scintia96;
    /// let cipher = Scintia96::new([0, 0, 0, 0]);
    /// let output = cipher.permute([1, 2, 3]);
    /// ```
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
        encrypt_step(k, x, y, z);
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
            assert_eq!(
                cipher.permute(input),
                expected,
                "Failed for key {:08x?}",
                key
            );
        }
    }
}
