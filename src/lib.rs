#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
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
//! - **`cipher`** (optional): Enables the `BlockCipher` implementation for server environments.
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
//!     // 96-bit block (3 x u32)
//!     let block = [0xdeadbeef, 0xcafebabe, 0xfacefeed];
//!     let permuted = PERMUTATION.permute(block);
//! }
//! ```

mod key_schedule;
mod scintia_96;
pub(crate) mod utils;

#[cfg(feature = "cipher")]
mod block_cipher;

pub use scintia_96::Scintia96;

#[cfg(feature = "cipher")]
pub use block_cipher::Scintia96Cipher;

pub(crate) const ROUNDS: u32 = 32;
