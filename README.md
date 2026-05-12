# 🌌 Scintia-96

[![Crates.io](https://img.shields.io/crates/v/scintia-96.svg)](https://crates.io/crates/scintia-96)
[![Documentation](https://docs.rs/scintia-96/badge.svg)](https://docs.rs/scintia-96)
[![License](https://img.shields.io/badge/license-BSD--2--Clause-blue.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

**Scintia-96** is a lightweight, keyed 96-bit permutation built for situations where you need guaranteed uniqueness for a 96-bit input. ⚡

---

## 🚀 Quick Start

Getting started is easy! Add it to your project:

```bash
cargo add scintia-96
```

### Features

- **`cipher`** (optional): Enables the `BlockCipher` implementation for server-side environments. This allows use with the `cipher` crate's traits, provides a precomputed key schedule for better performance, and **enables reversing (decrypting)** the permutation.

### Basic Usage

```rust
use scintia_96::Scintia96;

// 128-bit key (4 x u32)
const KEY: [u32; 4] = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
const PERMUTATION: Scintia96 = Scintia96::new(KEY);

fn main() {
    // 96-bit block (3 x u32)
    let block = [0xdeadbeef, 0xcafebabe, 0xfacefeed];
    
    // Permute it!
    let permuted = PERMUTATION.permute(block);
    
    println!("Original: {:x?}", block);
    println!("Permuted: {:x?}", permuted);
}
```

---

## 🎯 Why Scintia-96?

### The Use Cases

Scintia-96 is perfect for situations where you need to transform a 96-bit value while preserving its uniqueness:

- **Chip ID Masking:** Scintia-96 was originally created to **derive a USB serial number** from a 96-bit unique chip ID (such as those on STM32 microcontrollers) while preserving uniqueness.
- **ID Obfuscation:** MongoDB **BSON ObjectId**s are 96-bit values containing metadata such as timestamps and process identifiers. Scintia-96 can mask this information while preserving the useful properties of the ID. 🍃

While you could use a hash function, a **permutation** is often more suitable for cases like this. Since it's a one-to-one mapping, the output inherits the guaranteed uniqueness of the input, meaning there can never be a collision.

### Why a Keyed Permutation?

By using a key, you ensure that your derived result is unique to *your* use case. Even if other apps use Scintia-96 for the same purpose, their results will differ from yours. 🧂

---

## 🛠️ The Algorithm

Scintia-96 is based on the **Speck** block cipher (specifically Speck-64/128) with a few tweaks:
- **3-word GFN:** It operates on three 32-bit words using a Generalized Feistel Network. This native 32-bit design is highly efficient on common microcontrollers compared to 96-bit alternatives like Speck-96/144, which operate on 48-bit words. 🏎️
- **32 Rounds:** Since we have 3 lanes and leave one out each round, diffusion is slightly slower. We bumped the round count to 32 to compensate and ensure robust mixing. 🌪️

---

## 🔒 Security & Promises

I'm a developer, not a cryptographer, so I can't vouch for its resistance to cryptanalysis. But here’s what Scintia-96 brings to the table:

* ✅ **No Collisions:** It's a true permutation: 96-bit input, 96-bit output, with a one-to-one mapping.
* ✅ **Statistically Sound:** Passes avalanche and random distribution tests.
* ✅ **Hard to Reverse:** Prevents recovery of the input without the key.
* ❌ **Not Side-Channel Resistant:** If an attacker can perform side-channel analysis on your hardware, they can probably just as well extract the key from flash memory. 🤷‍♂️

### ⚠️ A Note on Safety

**Please don't use this to encrypt sensitive data.** For AEAD, KDF, or MAC use cases, stick to well-established primitives like **Xoodyak**. 

While we provide an optional `BlockCipher` implementation for convenience in server environments, Scintia-96 is designed primarily as a keyed permutation for identity masking. It has not undergone the rigorous cryptanalysis required for a general-purpose block cipher.

---

## 🔑 Key Generation

We provide a handy script to generate keys for your project:

```bash
python3 scripts/keygen.py
```

### Quick One-Liners

**Random Generation** (via `/dev/urandom`):
```bash
python3 -c "import os; d=os.urandom(16); print('const KEY: [u32; 4] = [%s];' % ', '.join('0x%08x' % int.from_bytes(d[i:i+4], 'big') for i in range(0, 16, 4)))"
```

**Deterministic Derivation** (via SHA-512):
```bash
key_material="my-unique-variant"
python3 -c "import hashlib; m='$key_material'; h=hashlib.sha512(('scintia-96:key:'+m).encode()).digest(); print('// derive_key(\"%s\")\nconst KEY: [u32; 4] = [%s];' % (m, ', '.join('0x%08x' % int.from_bytes(h[i:i+4], 'big') for i in range(0, 16, 4))))"
```

---

## 📜 License

This project is licensed under the **BSD 2-Clause License**. See [LICENSE.md](LICENSE.md) for the full text. 📄
