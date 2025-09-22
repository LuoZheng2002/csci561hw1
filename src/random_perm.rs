pub fn factorial(n: usize) -> usize {
    (1..=n).product()
}

pub fn unrank_permutation<T: Clone>(mut rank: usize, elements: &[T]) -> Vec<T> {
    let mut elems = elements.to_vec();
    let mut result = Vec::new();
    let n = elems.len();

    for i in (1..=n).rev() {
        let f = factorial(i - 1);
        let idx = rank / f;
        result.push(elems.remove(idx));
        rank %= f;
    }
    result
}

// pub fn feistel(index: usize, rounds: usize, key: usize, domain_size: usize) -> usize {
//     let mut l = index >> 16;
//     let mut r = index & 0xFFFF;

//     for i in 0..rounds {
//         let f = ((r.wrapping_mul(0x5bd1e995).wrapping_add(key + i)) & 0xFFFF) as usize;
//         let new_l = r;
//         let new_r = l ^ f;
//         l = new_l;
//         r = new_r;
//     }
//     let result = (l << 16) | r;
//     result % domain_size // modulo fold back into range
// }
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use std::num::Wrapping;
pub struct Feistel {
    rounds: usize,
    keys: Vec<u32>,
    bits: u32,
    mask: u32,
    n: u32,
}

impl Feistel {
    pub fn new(n: u32, seed: u64, rounds: usize) -> Self {
        let bits = (32 - (n - 1).leading_zeros()).max(1); // number of bits to cover n
        let mask = (1u32 << bits) - 1;

        // Generate round keys using a seeded RNG
        let mut rng = StdRng::seed_from_u64(seed);
        let keys = (0..rounds).map(|_| rng.next_u32()).collect();

        Self {
            rounds,
            keys,
            bits,
            mask,
            n,
        }
    }

    fn feistel(&self, mut x: u32) -> u32 {
        let half_bits = self.bits / 2;
        let l_mask = (1u32 << half_bits) - 1;

        let mut l = x & l_mask;
        let mut r = (x >> half_bits) & l_mask;

        for i in 0..self.rounds {
            let k = Wrapping(self.keys[i]);
            let f = Wrapping(r.wrapping_mul(0x5bd1e995).rotate_left(13)) ^ k;
            let new_l = r;
            let new_r = (l ^ f.0) & l_mask; // <-- FIXED HERE
            l = new_l;
            r = new_r;
        }

        (r << half_bits) | l
    }

    /// Maps `i ∈ 0..n` to a unique pseudorandom permutation of `0..n`
    pub fn permute(&self, i: u32) -> Option<u32> {
        if i >= self.n {
            return None;
        }

        // Cycle-walking to avoid out-of-range values
        let mut x = i;
        loop {
            let y = self.feistel(x) & self.mask;
            if y < self.n {
                return Some(y);
            }
            x = y; // walk to next step
        }
    }
}