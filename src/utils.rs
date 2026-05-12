//! Internal utilities for Scintia-96.

#[inline(always)]
pub(crate) fn bytes_to_words(bytes: &[u8; 12]) -> [u32; 3] {
    let mut words = [0u32; 3];
    for (i, chunk) in bytes.chunks_exact(4).enumerate() {
        words[i] = u32::from_le_bytes(chunk.try_into().unwrap());
    }
    words
}

#[inline(always)]
pub(crate) fn words_to_bytes(words: [u32; 3], bytes: &mut [u8; 12]) {
    for (i, &word) in words.iter().enumerate() {
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&word.to_le_bytes());
    }
}
