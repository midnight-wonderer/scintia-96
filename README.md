# Scintia-96

A keyed 96-bit permutation with a 128-bit key.

## What's the use case?

I can't talk for others, but the algorithm exists at all because I want to derive USB Serial Number from 96-bit unique chip IDs on STM32 microcontrollers.
Permutation is more suitable than hash, because the result will inherit the guaranteed uniqueness from the input.

## The algorithm

The algorithm is based on the Speck block cipher with a few modifications.
It run Speck-64/128 in 3-word Generalized Feistel Network and bump the round numbers to address slower diffusion from adding a third lane.

## The security

I am not a cryptographer and I can't vouch for cryptographic properties. But these are the promises:
- It will pass the avalanche and random distribution tests.
- It is a permutation function, it will map 96-bit input to 96-bit output one-to-one, no collision.
- It will prevent anyone from reverse engineering the input from the output without the key.
- It will not prevent side channel attacks; someone who could do that is better off breaking into the chip and read the key from Flash memory.
- The promises hold only when you don't mess up the key generation.

## Awareness

I hope no one use this to encrypt sensitive data. As we already have Xoodyak for AEAD, KDF, MAC, and what not. There are good reasons why there is no mainstream block ciphers operating on 96-bit block.

## The why

### Why starting with Speck-64/128? Why not just use Speck-96/144?

Word size: Speck64 has 32-bit word size.

### Why keyed permutation?

So that my USB Serial Number isn't the same as yours, if you happen to use this permutation for the same purpose.

### Why 32 rounds?

Sine we have 3 lanes and we leave one out in each round, only doing linear operation on the leftover, diffusion is slower. More round number is required to compensate.

## Usage

```bash
cargo add scintia-96
```

### Quick Start

```rust
use scintia_96::Scintia96;

const KEY: [u32; 4] = [0x01020304, 0x05060708, 0x090a0b0c, 0x0d0e0f10];
const PERMUTATION: Scintia96 = Scintia96::new(KEY);

fn main() {
    let block = [0xdeadbeef, 0xcafebabe, 0xfacefeed];
    let permuted = PERMUTATION.permute(block);
    println!("Permuted: {:?}", permuted);
}
```

## Keygen

Use the interactive script for guided key generation:

```bash
python3 scripts/keygen.py
```

For quick usage, you can also use these one-liners:

### Random Generation
Generate a unique, cryptographically secure random key using `/dev/urandom`:

```bash
python3 -c "import os; d=os.urandom(16); print('const KEY: [u32; 4] = [%s];' % ', '.join('0x%08x' % int.from_bytes(d[i:i+4], 'big') for i in range(0, 16, 4)))"
```

### Deterministic Derivation
Derive a key from a specific string (key material) using SHA-512. This is useful for creating specific variants or reproducible configurations:

```bash
key_material="my variant"
python3 -c "import hashlib; m='$key_material'; h=hashlib.sha512(('scintia-96:key:'+m).encode()).digest(); print('// derive_key(\"%s\")\nconst KEY: [u32; 4] = [%s];' % (m, ', '.join('0x%08x' % int.from_bytes(h[i:i+4], 'big') for i in range(0, 16, 4))))"
```
