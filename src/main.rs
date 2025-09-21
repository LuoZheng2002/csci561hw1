use std::rc::Rc;

use rust::{cover_tree::{CoverTree, Distance}, generator::ProblemGenerator, genetic::GeneticAlgorithm};


fn main() {
    let problem = ProblemGenerator::new(10, 100.0).generate_problem();

}
