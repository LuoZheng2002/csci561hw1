use std::{rc::Rc, time::Instant};

use rand::{SeedableRng, rngs::StdRng};
use rust::{
    adaptive_solver::adaptive_solve,
    cover_tree::{CoverTree, Distance},
    generator::ProblemGenerator,
    genetic::GeneticAlgorithm,
    problem::{self, Solution},
};

fn main() {
    let num_cities: u32 = 20;
    let problem_generator = ProblemGenerator::new(num_cities as usize, 20000.0, 42);
    let problem = Rc::new(problem_generator.generate_problem());
    let timer = Instant::now();
    let solution = adaptive_solve(problem.clone(), &timer);
    println!("Finished solving in {:?}", timer.elapsed());
    assert!(solution.is_valid(num_cities));
    println!("Solution length: {}", solution.total_distance());
    // let mut rng = StdRng::seed_from_u64(42);
    // for i in 0..4 {
    //     let random_shuffle_solution =
    //         Solution::from_random_shuffle(&problem, &mut rng);
    //     println!(
    //         "Random shuffle solution {} length: {}",
    //         i,
    //         random_shuffle_solution.total_distance()
    //     );
    // }
    // let mut timer = Instant::now();
    // let genetic_solution = GeneticAlgorithm::new(problem.clone(), 200, 20, 150).solve();
    // assert!(genetic_solution.is_valid(num_cities));
    // println!("Genetic algorithm time: {:?}", timer.elapsed());
    // println!(
    //     "Genetic algorithm solution length: {}",
    //     genetic_solution.total_distance()
    // );
    // for i in 0..10.min(num_cities as usize) {
    //     timer = Instant::now();
    //     let nearest_neighbor_solution =
    //         Solution::from_nearest_neighbor(&problem, i);
    //     assert!(nearest_neighbor_solution.is_valid(num_cities));
    //     println!("Nearest neighbor time: {:?}", timer.elapsed());
    //     println!(
    //         "Nearest neighbor solution length: {}",
    //         nearest_neighbor_solution.total_distance()
    //     );
    // }
}
