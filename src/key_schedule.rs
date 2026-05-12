/// Internal iterator for generating Scintia-96 round keys.
///
/// This implements the standard Speck key schedule update logic,
/// producing 32-bit round keys from a 128-bit master key.
pub(crate) struct SpeckKeySchedule {
    k: u32,
    l: [u32; 3],
    round: u32,
}

impl SpeckKeySchedule {
    /// Creates a new key schedule iterator from a 128-bit key.
    #[inline]
    pub(crate) fn new(key: [u32; 4]) -> Self {
        Self {
            k: key[0],
            l: [key[1], key[2], key[3]],
            round: 0,
        }
    }
}

impl Iterator for SpeckKeySchedule {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current_key = self.k;

        // Key Schedule Update (Speck standard logic)
        // alpha = 8, beta = 3
        let i = self.round;
        let l_idx = (i as usize) % 3;

        let new_l = (self.k.wrapping_add(self.l[l_idx].rotate_right(8))) ^ i;
        let new_k = self.k.rotate_left(3) ^ new_l;

        self.l[l_idx] = new_l;
        self.k = new_k;
        self.round += 1;

        Some(current_key)
    }
}

#[inline(always)]
pub(crate) fn encrypt_step(k: u32, x: &mut u32, y: &mut u32, z: &mut u32) {
    *x = (x.rotate_right(8).wrapping_add(*y)) ^ k;
    *y = y.rotate_left(3) ^ *x;
    *z ^= *y;
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn decrypt_step(k: u32, x: &mut u32, y: &mut u32, z: &mut u32) {
    *z ^= *y;
    *y = (*y ^ *x).rotate_right(3);
    *x = ((*x ^ k).wrapping_sub(*y)).rotate_left(8);
}
