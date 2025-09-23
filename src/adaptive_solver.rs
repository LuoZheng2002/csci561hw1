use std::{rc::Rc, time::Instant};

use crate::{
    genetic::GeneticAlgorithm,
    problem::{Problem, Solution},
};

pub fn adaptive_solve(problem: Rc<Problem>, timer: &Instant) -> Solution {
    let num_cities = problem.cities.len();
    let population_size: usize = 200.max(num_cities);
    let extra_population_size: usize = 200.max(num_cities);
    const MARGIN: u64 = 2;
    const CLASS_1_TIME_LIMIT: u64 = 60;
    const CLASS_2_TIME_LIMIT: u64 = 75;
    const CLASS_3_TIME_LIMIT: u64 = 120;
    const CLASS_4_TIME_LIMIT: u64 = 300;
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
