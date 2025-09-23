use rust::generator::ProblemGenerator;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[test]
fn test_generate() {
    let num_cities = 500;
    let generator = ProblemGenerator::new(num_cities, 10000.0, 42);
    let problem = generator.generate_problem();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .append(false)
        .open("../test_cases/test_case11.txt")
        .unwrap();
    writeln!(file, "{}", problem.cities.len()).unwrap();
    for city in problem.cities {
        writeln!(file, "{} {} {}", city.x, city.y, city.z).unwrap();
    }
}
