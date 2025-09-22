use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use rust::random_perm::{Feistel, factorial, unrank_permutation};

    use super::*;

    #[test]
    fn test_permutation_generator_n3() {
        let items = vec![1, 2, 3];
        let n = items.len();
        let total = factorial(n);
        let rounds = 4;

        let mut seen = HashSet::new();
        let feistel = Feistel::new(total as u32, 42, rounds);

        for i in 0..total {
            let idx = feistel.permute(i as u32).unwrap();
            println!("permuted index: {}", idx);
            let perm = unrank_permutation(idx as usize, &items);
            println!("Generated permutation: {:?}", perm);
            assert!(
                seen.insert(perm.clone()),
                "Duplicate permutation: {:?}",
                perm
            );
        }

        assert_eq!(seen.len(), total, "Not all permutations were generated");
    }

    #[test]
    fn test_permutation_generator_n4() {
        let items = vec![1, 2, 3, 4];
        let n = items.len();
        let total = factorial(n);
        let rounds = 4;

        let mut seen = HashSet::new();
        let feistel = Feistel::new(total as u32, 42, rounds);

        for i in 0..total {
            let idx = feistel.permute(i as u32).unwrap();
            let perm = unrank_permutation(idx as usize, &items);
            assert!(
                seen.insert(perm.clone()),
                "Duplicate permutation: {:?}",
                perm
            );
        }
        assert_eq!(seen.len(), total, "Not all permutations were generated");
    }
}
