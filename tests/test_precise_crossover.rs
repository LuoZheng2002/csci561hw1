use std::rc::Rc;

use rust::{
    genetic::GeneticAlgorithm,
    problem::{City, Problem, Solution},
};

#[test]
fn test_precise_crossover() {
    let problem = Rc::new(Problem {
        cities: vec![
            City::new(0, 0, 0),
            City::new(1, 0, 1),
            City::new(1, 1, 1),
            City::new(0, 1, 0),
            City::new(0, 0, 0),
            City::new(0, 1, 0),
        ],
    });
    let solution1 = Solution::new(vec![0, 1, 2, 3, 4, 5], Rc::downgrade(&problem), None);
    let solution2 = Solution::new(vec![0, 1, 3, 2, 4, 5], Rc::downgrade(&problem), None);
    GeneticAlgorithm::precise_crossover(&solution1, &solution2, &mut |child: &Solution| {
        println!("found crossover");
        assert!(child.is_valid(6));
        // assert_eq!(child.total_distance(), 6.0);
        true
    });
}
