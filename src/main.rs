use std::rc::Rc;

use rust::{
    cover_tree::{CoverTree, Distance},
    generator::ProblemGenerator,
    genetic::GeneticAlgorithm,
    problem::{self, Solution},
};

fn main() {
    let problem_generator = ProblemGenerator::new(10, 100.0);
    let problem = Rc::new(problem_generator.generate_problem());
    let mut solution_id_iter = 0..;
    let brute_force_solution = Solution::from_brute_force(&problem, &mut solution_id_iter);
    println!(
        "Brute force solution length: {}",
        brute_force_solution.total_distance()
    );
    let nearest_neighbor_solution =
        Solution::from_nearest_neighbor(&problem, &mut solution_id_iter);
    println!(
        "Nearest neighbor solution length: {}",
        nearest_neighbor_solution.total_distance()
    );
}
