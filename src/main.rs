use std::rc::Rc;

use crate::{generator::ProblemGenerator, genetic::GeneticAlgorithm};




mod generator;
mod problem;
mod genetic;
mod kdtree;
mod cover_tree;
mod cover_tree_chatgpt;


fn main() {
    let problem_generator = ProblemGenerator::new(100, 500.0);
    for _ in 0..5{
        let problem = problem_generator.generate_problem();
    let genetic_algorithm = GeneticAlgorithm::new(Rc::new(problem), 40, 20, 40);
    let best_solution = genetic_algorithm.solve();
    println!("Best solution found: id: {}, total_length: {}", best_solution.id, best_solution.total_length());    
    }    
}
