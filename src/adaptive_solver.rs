use std::{rc::Rc, time::Instant};

use crate::{
    genetic::GeneticAlgorithm,
    problem::{Problem, Solution},
};

pub fn adaptive_solve(problem: Rc<Problem>, timer: &Instant) -> Solution {
    let num_cities = problem.cities.len();
    let population_size: usize = (num_cities * 2).max(2000);
    let extra_population_size: usize = (num_cities * 2).max(2000);
    const MARGIN: u64 = 3;
    const CLASS_1_TIME_LIMIT: u64 = 10;
    const CLASS_2_TIME_LIMIT: u64 = 15;
    const CLASS_3_TIME_LIMIT: u64 = 15;
    const CLASS_4_TIME_LIMIT: u64 = 60;
    let solution = match num_cities {
        0..10 => Solution::from_brute_force(&problem),
        10..50 => GeneticAlgorithm::new(problem, population_size, extra_population_size)
            .solve(timer, CLASS_1_TIME_LIMIT - MARGIN),
        50..100 => GeneticAlgorithm::new(problem, population_size, extra_population_size)
            .solve(timer, CLASS_1_TIME_LIMIT - MARGIN),
        100..200 => GeneticAlgorithm::new(problem, population_size, extra_population_size)
            .solve(timer, CLASS_2_TIME_LIMIT - MARGIN),
        200..500 => GeneticAlgorithm::new(problem, population_size, extra_population_size)
            .solve(timer, CLASS_3_TIME_LIMIT - MARGIN),
        500.. => GeneticAlgorithm::new(problem, population_size, extra_population_size)
            .solve(timer, CLASS_4_TIME_LIMIT - MARGIN),
    };
    solution
}

// brute force, genetic, nearest neighbor, nearest neighbor with different starting points,

// always use nearest neighbor first?
// time out?
