use std::cell::RefCell;

use rand::distr::Uniform;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rust::cover_tree::{CoverTree, Distance};
use rust::generator::ProblemGenerator;

// pub struct Problem1DGenerator{
//     pub num_numbers: usize,
//     pub max_value: u32,
//     pub rng: RefCell<StdRng>,
// }

// impl Problem1DGenerator{
//     pub fn new(num_numbers: usize, max_value: u32) -> Self {
//         let rng = StdRng::seed_from_u64(42);
//         Self {
//             num_numbers,
//             max_value,
//             rng: RefCell::new(rng),
//         }
//     }
//     pub fn generate_problem(&self) -> Vec<u32>{
//         let mut rng = self.rng.borrow_mut();
//         let dist = Uniform::new(0, self.max_value).unwrap();
//         let numbers = (0..self.num_numbers)
//             .map(|_| rng.sample(&dist))
//             .collect();
//         numbers
//     }
// }

#[test]
fn pressure_test_cover_tree_nearest_neighbor() {
    let num_points = 10;
    let num_queries = 1;

    let mut problem: Vec<u32> = (0..num_points).collect();
    let mut rng = StdRng::seed_from_u64(42);
    problem.shuffle(&mut rng);

    let mut tree = CoverTree::new();

    for num in &problem {
        tree.insert(num.clone());
        if let Err(e) = tree.assert_valid_cover_tree() {
            println!("Cover tree failed validation after inserting: {}", e);
            tree.print();
            panic!();
        }
    }
    println!("Successfully inserted all points into the cover tree.");

    tree.print();

    println!("Finished building the cover tree.");

    // Validate structure (optional)
    if let Err(e) = tree.assert_valid_cover_tree() {
        panic!("Cover tree failed validation: {}", e);
    }

    let mut num_mismatches = 0;

    for (i, query) in problem.iter().take(num_queries).enumerate() {
        let brute_result = problem
            .iter()
            .filter(|p| *p != query)
            .map(|p| (p, p.distance(query)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        tree.remove(query);
        if let Err(e) = tree.assert_valid_cover_tree() {
            println!("Cover tree failed validation after removing: {}", e);
            tree.print();
            panic!();
        }
        println!("Tree after remove:");
        tree.print();
        let tree_result = tree.nearest_neighbor(query).unwrap();

        let dist_diff = (tree_result.1 - brute_result.1).abs();
        let same_point = &tree_result.0 == brute_result.0;

        if dist_diff > 1e-4 || !same_point {
            println!(
                "Mismatch at query {}: query num: {:?}, brute {:?} vs tree {:?}, dist diff {}",
                i, query, brute_result, tree_result, dist_diff
            );
            num_mismatches += 1;
        }
    }

    assert_eq!(num_mismatches, 0, "Found {} mismatches!", num_mismatches);
}
