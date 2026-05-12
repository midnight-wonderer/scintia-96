use cipher::{
    BlockCipherDecBackend, BlockCipherDecClosure, BlockCipherDecrypt, BlockCipherEncBackend,
    BlockCipherEncClosure, BlockCipherEncrypt, BlockSizeUser, KeyInit, KeySizeUser,
    ParBlocksSizeUser,
    consts::{U1, U12, U16},
    inout::InOut,
};

use crate::key_schedule::{SpeckKeySchedule, decrypt_step, encrypt_step};
use crate::{ROUNDS, Scintia96Permuter, Scintia96Unpermuter, utils};

type InoutBlock<'a, 'b, T> = InOut<'a, 'b, cipher::Block<T>>;

/// A Scintia-96 instance optimized for server environments with precomputed key schedule.
///
/// Implements the `BlockCipher` trait from the `cipher` crate.
///
/// ## Examples
///
/// ```rust
/// use scintia_96::Scintia96Cipher;
///
/// let key = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
/// let cipher = Scintia96Cipher::new(key);
///
/// let mut block = [0u8; 12];
/// cipher.permute_block(&mut block);
/// cipher.unpermute_block(&mut block);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Scintia96Cipher {
    round_keys: [u32; ROUNDS as usize],
}

impl Scintia96Cipher {
    /// Creates a new Scintia-96 cipher instance from a 128-bit key.
    ///
    /// This constructor precomputes the entire key schedule (32 round keys),
    /// allowing for much faster encryption and decryption in environments
    /// where memory is less constrained than a typical microcontroller.
    pub fn new(key: [u32; 4]) -> Self {
        let mut k_schedule = [0u32; ROUNDS as usize];
        let mut schedule_iter = SpeckKeySchedule::new(key);
        for key_word in &mut k_schedule {
            *key_word = schedule_iter.next().unwrap();
        }

        Self {
            round_keys: k_schedule,
        }
    }

    /// Permutes a 96-bit block represented as three 32-bit words.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use scintia_96::Scintia96Cipher;
    /// # let key = [0u32; 4];
    /// # let cipher = Scintia96Cipher::new(key);
    /// let block = [0xdeadbeef, 0xcafebabe, 0xfacefeed];
    /// let permuted = cipher.permute(block);
    /// ```
    #[inline]
    pub fn permute(&self, block: [u32; 3]) -> [u32; 3] {
        Scintia96Permuter::permute(self, block)
    }

    /// Permutes a 96-bit block in place.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use scintia_96::Scintia96Cipher;
    /// # let key = [0u32; 4];
    /// # let cipher = Scintia96Cipher::new(key);
    /// let mut block = [0u8; 12];
    /// cipher.permute_block(&mut block);
    /// ```
    #[inline]
    pub fn permute_block(&self, block: &mut [u8; 12]) {
        Scintia96Permuter::permute_block(self, block)
    }

    /// Un-permutes a 96-bit block represented as three 32-bit words.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use scintia_96::Scintia96Cipher;
    /// # let key = [0u32; 4];
    /// # let cipher = Scintia96Cipher::new(key);
    /// let block = [0xdeadbeef, 0xcafebabe, 0xfacefeed];
    /// let unpermuted = cipher.unpermute(block);
    /// ```
    #[inline]
    pub fn unpermute(&self, block: [u32; 3]) -> [u32; 3] {
        Scintia96Unpermuter::unpermute(self, block)
    }

    /// Un-permutes a 96-bit block in place.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use scintia_96::Scintia96Cipher;
    /// # let key = [0u32; 4];
    /// # let cipher = Scintia96Cipher::new(key);
    /// let mut block = [0u8; 12];
    /// cipher.unpermute_block(&mut block);
    /// ```
    #[inline]
    pub fn unpermute_block(&self, block: &mut [u8; 12]) {
        Scintia96Unpermuter::unpermute_block(self, block)
    }
}

impl Scintia96Permuter for Scintia96Cipher {
    /// Permutes a 96-bit block using the precomputed key schedule.
    ///
    /// This is the inherent equivalent of `Scintia96::permute`.
    fn permute(&self, block: [u32; 3]) -> [u32; 3] {
        Scintia96EncryptBackend {
            round_keys: &self.round_keys,
        }
        .permute(block)
    }
}

impl Scintia96Unpermuter for Scintia96Cipher {
    /// Un-permutes a 96-bit block using the precomputed key schedule.
    fn unpermute(&self, block: [u32; 3]) -> [u32; 3] {
        Scintia96DecryptBackend {
            round_keys: &self.round_keys,
        }
        .unpermute(block)
    }
}

impl KeySizeUser for Scintia96Cipher {
    type KeySize = U16;
}

impl KeyInit for Scintia96Cipher {
    fn new(key: &cipher::Key<Self>) -> Self {
        let mut key_words = [0u32; 4];
        for (i, chunk) in key.chunks_exact(4).enumerate() {
            key_words[i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        Self::new(key_words)
    }
}

impl BlockSizeUser for Scintia96Cipher {
    type BlockSize = U12;
}

impl BlockCipherEncrypt for Scintia96Cipher {
    fn encrypt_with_backend(&self, f: impl BlockCipherEncClosure<BlockSize = Self::BlockSize>) {
        f.call(&Scintia96EncryptBackend {
            round_keys: &self.round_keys,
        })
    }
}

impl BlockCipherDecrypt for Scintia96Cipher {
    fn decrypt_with_backend(&self, f: impl BlockCipherDecClosure<BlockSize = Self::BlockSize>) {
        f.call(&Scintia96DecryptBackend {
            round_keys: &self.round_keys,
        })
    }
}

struct Scintia96EncryptBackend<'a> {
    round_keys: &'a [u32; ROUNDS as usize],
}

impl<'a> Scintia96Permuter for Scintia96EncryptBackend<'a> {
    fn permute(&self, block: [u32; 3]) -> [u32; 3] {
        let mut a = block[0];
        let mut b = block[1];
        let mut c = block[2];

        for i in 0..ROUNDS as usize {
            let k = self.round_keys[i];
            match i % 3 {
                0 => encrypt_step(k, &mut a, &mut b, &mut c),
                1 => encrypt_step(k, &mut b, &mut c, &mut a),
                _ => encrypt_step(k, &mut c, &mut a, &mut b),
            }
        }

        match ROUNDS % 3 {
            1 => [b, c, a],
            2 => [c, a, b],
            _ => [a, b, c],
        }
    }
}

impl<'a> BlockSizeUser for Scintia96EncryptBackend<'a> {
    type BlockSize = U12;
}

impl<'a> ParBlocksSizeUser for Scintia96EncryptBackend<'a> {
    type ParBlocksSize = U1;
}

impl<'a> BlockCipherEncBackend for Scintia96EncryptBackend<'a> {
    #[inline]
    fn encrypt_block(&self, mut block: InoutBlock<'_, '_, Self>) {
        let words = utils::bytes_to_words(block.get_in().as_ref());
        let out = self.permute(words);
        utils::words_to_bytes(out, block.get_out().as_mut());
    }
}

struct Scintia96DecryptBackend<'a> {
    round_keys: &'a [u32; ROUNDS as usize],
}

impl<'a> Scintia96Unpermuter for Scintia96DecryptBackend<'a> {
    fn unpermute(&self, block: [u32; 3]) -> [u32; 3] {
        let (mut a, mut b, mut c) = match ROUNDS % 3 {
            1 => (block[2], block[0], block[1]),
            2 => (block[1], block[2], block[0]),
            _ => (block[0], block[1], block[2]),
        };

        for i in (0..ROUNDS as usize).rev() {
            let k = self.round_keys[i];
            match i % 3 {
                0 => decrypt_step(k, &mut a, &mut b, &mut c),
                1 => decrypt_step(k, &mut b, &mut c, &mut a),
                _ => decrypt_step(k, &mut c, &mut a, &mut b),
            }
        }

        [a, b, c]
    }
}

impl<'a> BlockSizeUser for Scintia96DecryptBackend<'a> {
    type BlockSize = U12;
}

impl<'a> ParBlocksSizeUser for Scintia96DecryptBackend<'a> {
    type ParBlocksSize = U1;
}

impl<'a> BlockCipherDecBackend for Scintia96DecryptBackend<'a> {
    #[inline]
    fn decrypt_block(&self, mut block: InoutBlock<'_, '_, Self>) {
        let words = utils::bytes_to_words(block.get_in().as_ref());
        let out = self.unpermute(words);
        utils::words_to_bytes(out, block.get_out().as_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scintia96;

    #[test]
    fn test_scintia96cipher_consistency() {
        let key = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
        let cipher = Scintia96Cipher::new(key);

        let mut block = [0u8; 12];
        let original = block;

        cipher.permute_block(&mut block);
        assert_ne!(block, original);

        cipher.unpermute_block(&mut block);
        assert_eq!(block, original);
    }

    #[test]
    fn test_scintia96cipher_matches_scintia96() {
        let key_u32: [u32; 4] = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
        let scintia = Scintia96::new(key_u32);
        let cipher = Scintia96Cipher::new(key_u32);

        let input_u32: [u32; 3] = [0xdeadbeef, 0xcafebabe, 0xfacefeed];
        let mut input_bytes = [0u8; 12];
        for (i, &w) in input_u32.iter().enumerate() {
            input_bytes[i * 4..(i + 1) * 4].copy_from_slice(&w.to_le_bytes());
        }

        let expected_u32 = scintia.permute(input_u32);
        let mut expected_bytes = [0u8; 12];
        for (i, &w) in expected_u32.iter().enumerate() {
            expected_bytes[i * 4..(i + 1) * 4].copy_from_slice(&w.to_le_bytes());
        }

        let mut block = input_bytes;
        cipher.permute_block(&mut block);

        assert_eq!(block, expected_bytes);
    }
}
