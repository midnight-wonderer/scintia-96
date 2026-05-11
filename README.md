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

You can generate a compatible 128-bit key for use in your Rust code using the following one-liners.

### Random Generation
Generate a unique, cryptographically secure random key using `/dev/urandom`:

```bash
printf "const KEY: [u32; 4] = [%s, %s, %s, %s];\n" $(head -c 16 /dev/urandom | od -An -vtx4 | awk '{for(i=1;i<=NF;i++) print "0x"$i}')
```

### Deterministic Derivation
Derive a key from a specific string (key material) using SHA-512. This is useful for creating specific variants or reproducible configurations:

```bash
key_material="my variant"
(echo -n "scintia-96:key:$key_material" | sha512sum 2>/dev/null || echo -n "scintia-96:key:$key_material" | shasum -a 512) | awk -v w="$key_material" '{h=$1; printf "// derive_key(\"%s\")\nconst KEY: [u32; 4] = [0x%s, 0x%s, 0x%s, 0x%s];\n", w, substr(h,1,8), substr(h,9,8), substr(h,17,8), substr(h,25,8)}'
```
