use cipher::{
    BlockBackend, BlockCipher, BlockDecrypt, BlockEncrypt, BlockSizeUser, KeyInit, KeySizeUser,
    ParBlocksSizeUser,
    consts::{U1, U12, U16},
    inout::InOut,
};

use crate::ROUNDS;
use crate::key_schedule::{SpeckKeySchedule, decrypt_step, encrypt_step};

type InoutBlock<'a, 'b, T> = InOut<'a, 'b, cipher::Block<T>>;

/// A Scintia-96 instance optimized for server environments with precomputed key schedule.
///
/// Implements the `BlockCipher` trait from the `cipher` crate.
#[derive(Debug, Clone, Copy)]
pub struct Scintia96Cipher {
    round_keys: [u32; ROUNDS as usize],
}

impl KeySizeUser for Scintia96Cipher {
    type KeySize = U16;
}

impl KeyInit for Scintia96Cipher {
    /// Creates a new Scintia-96 cipher instance from a 128-bit key.
    ///
    /// This constructor precomputes the entire key schedule (32 round keys),
    /// allowing for much faster encryption and decryption in environments
    /// where memory is less constrained than a typical microcontroller.
    fn new(key: &cipher::Key<Self>) -> Self {
        let mut k_schedule = [0u32; ROUNDS as usize];
        let mut key_words = [0u32; 4];
        for (i, chunk) in key.chunks_exact(4).enumerate() {
            key_words[i] = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        let mut schedule_iter = SpeckKeySchedule::new(key_words);
        for i in 0..ROUNDS as usize {
            k_schedule[i] = schedule_iter.next().unwrap();
        }

        Self {
            round_keys: k_schedule,
        }
    }
}

impl BlockSizeUser for Scintia96Cipher {
    type BlockSize = U12;
}

impl BlockCipher for Scintia96Cipher {}

impl BlockEncrypt for Scintia96Cipher {
    fn encrypt_with_backend(&self, f: impl cipher::BlockClosure<BlockSize = Self::BlockSize>) {
        f.call(&mut Scintia96EncryptBackend {
            round_keys: &self.round_keys,
        })
    }
}

impl BlockDecrypt for Scintia96Cipher {
    fn decrypt_with_backend(&self, f: impl cipher::BlockClosure<BlockSize = Self::BlockSize>) {
        f.call(&mut Scintia96DecryptBackend {
            round_keys: &self.round_keys,
        })
    }
}

struct Scintia96EncryptBackend<'a> {
    round_keys: &'a [u32; ROUNDS as usize],
}

impl<'a> BlockSizeUser for Scintia96EncryptBackend<'a> {
    type BlockSize = U12;
}

impl<'a> ParBlocksSizeUser for Scintia96EncryptBackend<'a> {
    type ParBlocksSize = U1;
}

impl<'a> BlockBackend for Scintia96EncryptBackend<'a> {
    #[inline]
    fn proc_block(&mut self, mut block: InoutBlock<'_, '_, Self>) {
        let b = block.get_in();
        let mut a = u32::from_le_bytes(b[0..4].try_into().unwrap());
        let mut b_word = u32::from_le_bytes(b[4..8].try_into().unwrap());
        let mut c = u32::from_le_bytes(b[8..12].try_into().unwrap());

        for i in 0..ROUNDS as usize {
            let k = self.round_keys[i];
            match i % 3 {
                0 => encrypt_step(k, &mut a, &mut b_word, &mut c),
                1 => encrypt_step(k, &mut b_word, &mut c, &mut a),
                _ => encrypt_step(k, &mut c, &mut a, &mut b_word),
            }
        }

        let out = match ROUNDS % 3 {
            1 => [b_word, c, a],
            2 => [c, a, b_word],
            _ => [a, b_word, c],
        };

        let out_block = block.get_out();
        out_block[0..4].copy_from_slice(&out[0].to_le_bytes());
        out_block[4..8].copy_from_slice(&out[1].to_le_bytes());
        out_block[8..12].copy_from_slice(&out[2].to_le_bytes());
    }
}

struct Scintia96DecryptBackend<'a> {
    round_keys: &'a [u32; ROUNDS as usize],
}

impl<'a> BlockSizeUser for Scintia96DecryptBackend<'a> {
    type BlockSize = U12;
}

impl<'a> ParBlocksSizeUser for Scintia96DecryptBackend<'a> {
    type ParBlocksSize = U1;
}

impl<'a> BlockBackend for Scintia96DecryptBackend<'a> {
    #[inline]
    fn proc_block(&mut self, mut block: InoutBlock<'_, '_, Self>) {
        let b = block.get_in();
        let word0 = u32::from_le_bytes(b[0..4].try_into().unwrap());
        let word1 = u32::from_le_bytes(b[4..8].try_into().unwrap());
        let word2 = u32::from_le_bytes(b[8..12].try_into().unwrap());

        let (mut a, mut b_word, mut c) = match ROUNDS % 3 {
            1 => (word2, word0, word1),
            2 => (word1, word2, word0),
            _ => (word0, word1, word2),
        };

        for i in (0..ROUNDS as usize).rev() {
            let k = self.round_keys[i];
            match i % 3 {
                0 => decrypt_step(k, &mut a, &mut b_word, &mut c),
                1 => decrypt_step(k, &mut b_word, &mut c, &mut a),
                _ => decrypt_step(k, &mut c, &mut a, &mut b_word),
            }
        }

        let out_block = block.get_out();
        out_block[0..4].copy_from_slice(&a.to_le_bytes());
        out_block[4..8].copy_from_slice(&b_word.to_le_bytes());
        out_block[8..12].copy_from_slice(&c.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scintia96;

    #[test]
    fn test_scintia96cipher_consistency() {
        use cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};

        let key_bytes = [0u8; 16];
        let key = GenericArray::from_slice(&key_bytes);
        let cipher = Scintia96Cipher::new(key);

        let mut block = [0u8; 12];
        let original = block;

        cipher.encrypt_block(GenericArray::from_mut_slice(&mut block));
        assert_ne!(block, original);

        cipher.decrypt_block(GenericArray::from_mut_slice(&mut block));
        assert_eq!(block, original);
    }

    #[test]
    fn test_scintia96cipher_matches_scintia96() {
        use cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};

        let key_u32: [u32; 4] = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
        let mut key_bytes = [0u8; 16];
        for (i, &k) in key_u32.iter().enumerate() {
            key_bytes[i * 4..(i + 1) * 4].copy_from_slice(&k.to_le_bytes());
        }

        let scintia = Scintia96::new(key_u32);
        let cipher = Scintia96Cipher::new(GenericArray::from_slice(&key_bytes));

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
        cipher.encrypt_block(GenericArray::from_mut_slice(&mut block));

        assert_eq!(block, expected_bytes);
    }
}
