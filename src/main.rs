use std::{rc::Rc, time::Instant};

use rand::{SeedableRng, rngs::StdRng};
use rust::{
    adaptive_solver::adaptive_solve,
    cover_tree::{CoverTree, Distance},
    generator::ProblemGenerator,
    genetic::GeneticAlgorithm,
    problem::{self, City, Problem, Solution},
};

// fn main() {
//     let num_cities: u32 = 10;
//     let problem_generator = ProblemGenerator::new(num_cities as usize, 20000.0, 43);
//     let problem = Rc::new(problem_generator.generate_problem());
//     let timer = Instant::now();
//     let solution = adaptive_solve(problem.clone(), &timer);
//     println!("Finished solving in {:?}", timer.elapsed());
//     assert!(solution.is_valid(num_cities));
//     println!("Solution length: {}", solution.total_distance());
//     // let mut rng = StdRng::seed_from_u64(42);
//     // for i in 0..4 {
//     //     let random_shuffle_solution =
//     //         Solution::from_random_shuffle(&problem, &mut rng);
//     //     println!(
//     //         "Random shuffle solution {} length: {}",
//     //         i,
//     //         random_shuffle_solution.total_distance()
//     //     );
//     // }
//     // let mut timer = Instant::now();
//     // let genetic_solution = GeneticAlgorithm::new(problem.clone(), 200, 20, 150).solve();
//     // assert!(genetic_solution.is_valid(num_cities));
//     // println!("Genetic algorithm time: {:?}", timer.elapsed());
//     // println!(
//     //     "Genetic algorithm solution length: {}",
//     //     genetic_solution.total_distance()
//     // );
//     // for i in 0..10.min(num_cities as usize) {
//     //     timer = Instant::now();
//     //     let nearest_neighbor_solution =
//     //         Solution::from_nearest_neighbor(&problem, i);
//     //     assert!(nearest_neighbor_solution.is_valid(num_cities));
//     //     println!("Nearest neighbor time: {:?}", timer.elapsed());
//     //     println!(
//     //         "Nearest neighbor solution length: {}",
//     //         nearest_neighbor_solution.total_distance()
//     //     );
//     // }
// }

use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let input_file = File::open("../test_cases/test_case11.txt").expect("Failed to open input file");
    let mut reader = BufReader::new(input_file);

    let mut first_line = String::new();
    reader.read_line(&mut first_line).unwrap();
    let n: u32 = first_line.trim().parse().unwrap();

    println!("Number of cities: {}", n);

    let mut cities = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let coords: Vec<u32> = line
            .split_whitespace()
            .map(|s| s.parse::<u32>().unwrap())
            .collect();
        let (x, y, z) = (coords[0], coords[1], coords[2]);
        cities.push(City::new(x, y, z)); // Assumes City::new(x, y, z) exists
    }

    let problem = Rc::new(Problem::new(cities)); // Assumes Problem::new(cities) exists

    let mut timer = Instant::now(); // Assumes Timer::new() and start() exist

    let solution = adaptive_solve(Rc::clone(&problem), &timer);

    let mut output_file = File::create("output.txt").expect("Failed to create output file");

    writeln!(output_file, "{}", solution.total_distance()).unwrap();

    for &city_index in &solution.order_without_loop {
        let city = &problem.cities[city_index as usize];
        writeln!(output_file, "{} {} {}", city.x, city.y, city.z).unwrap();
    }

    if let Some(&first_index) = solution.order_without_loop.first() {
        let city = &problem.cities[first_index as usize];
        writeln!(output_file, "{} {} {}", city.x, city.y, city.z).unwrap();
    }

    println!("Time used: {} seconds", timer.elapsed().as_secs());
    println!("Best distance: {}", solution.total_distance());
}
