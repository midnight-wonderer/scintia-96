#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Scintia-96
//!
//! A lightweight, keyed 96-bit permutation based on the Speck cipher.
//!
//! Scintia-96 is designed for use cases where you need a one-to-one mapping of a 96-bit input
//! (such as a hardware unique ID) to a 96-bit output (such as a USB serial number),
//! ensuring that uniqueness is preserved without collisions.
//!
//!
//! ## Features
//!
//! - **`cipher`** (optional): Enables the RustCrypto cipher traits implementation for server environments.
//!   This includes the `Scintia96Cipher` struct which precomputes the key schedule for efficiency
//!   and enables **reversing (decrypting)** the permutation.
//!
//! ## Example
//!
//! ```rust
//! use scintia_96::Scintia96;
//!
//! // 128-bit key (4 x u32)
//! const KEY: [u32; 4] = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
//! const PERMUTATION: Scintia96 = Scintia96::new(KEY);
//!
//! fn main() {
//!     // 96-bit block as raw bytes
//!     let mut block = [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xfe, 0xed];
//!     PERMUTATION.permute_block(&mut block);
//! }
//! ```
//!
//! For more advanced usage or server-side environments, enable the `cipher` feature to use `Scintia96Cipher`.

mod key_schedule;
mod scintia_96;
pub(crate) mod utils;

#[cfg(feature = "cipher")]
mod block_cipher;

pub use scintia_96::Scintia96;

#[cfg(feature = "cipher")]
pub use block_cipher::Scintia96Cipher;

/// A trait for types that can perform the Scintia-96 96-bit keyed permutation.
///
/// Most users should use the inherent methods on [`Scintia96`] or [`Scintia96Cipher`]
/// instead of this trait directly.
pub trait Scintia96Permuter {
    /// Permutes a 96-bit block represented as three 32-bit words.
    fn permute(&self, block: [u32; 3]) -> [u32; 3];

    /// Permutes a 96-bit block in place.
    fn permute_block(&self, block: &mut [u8; 12]) {
        let words = utils::bytes_to_words(block);
        let out = self.permute(words);
        utils::words_to_bytes(out, block);
    }
}

/// A trait for types that can perform the inverse Scintia-96 96-bit keyed permutation.
///
/// Most users should use the inherent methods on [`Scintia96Cipher`] instead of this
/// trait directly.
pub trait Scintia96Unpermuter {
    /// Un-permutes a 96-bit block represented as three 32-bit words.
    fn unpermute(&self, block: [u32; 3]) -> [u32; 3];

    /// Un-permutes a 96-bit block in place.
    fn unpermute_block(&self, block: &mut [u8; 12]) {
        let words = utils::bytes_to_words(block);
        let out = self.unpermute(words);
        utils::words_to_bytes(out, block);
    }
}

pub(crate) const ROUNDS: u32 = 32;
