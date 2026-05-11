# Scintia-96

A keyed 96-bit permutation library for Rust, designed for `no_std` environments.

## Features

- **96-bit State**: Operates on 3 x `u32` words.
- **128-bit Key**: Uses a 4 x `u32` key.
- **Speck-based**: Leverages the ARX (Addition-Rotation-XOR) design of the Speck block cipher.
- **no_std**: Zero dependencies, suitable for embedded systems.
- **On-demand Key Schedule**: Generates round keys on the fly to minimize memory usage.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
scintia-96 = { path = "..." }
```

### Example

```rust
use scintia_96::Scintia96;

fn main() {
    let key = [0x01234567, 0x89abcdef, 0x01234567, 0x89abcdef];
    let block = [0xdeadbeef, 0xcafebabe, 0xfacefeed];

    let cipher = Scintia96::new(key);
    let ciphertext = cipher.permute(block);
    
    println!("Ciphertext: {:?}", ciphertext);
}
```

## Algorithm Details

The permutation uses 32 rounds. Each round consists of:
1. A standard Speck round function applied to the first two words.
2. XORing the result of the second word into the third word.
3. A word-level left rotation (shuffle) of the entire state.

Round keys are generated using the standard Speck-128 key schedule logic, adapted for the 32 rounds of this permutation.

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
