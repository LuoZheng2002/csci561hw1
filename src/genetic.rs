use core::f32;
use std::{cell::RefCell, cmp::Ordering, collections::BTreeSet, rc::Rc, time::Instant};

use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

use crate::problem::{Population, Problem, RcKey, Solution};

pub struct GeneticAlgorithm {
    pub problem: Rc<Problem>,
    pub population_size: usize,
    pub extra_population_size: usize,
    rng: RefCell<StdRng>,
}

impl GeneticAlgorithm {
    pub fn new(
        problem: Rc<Problem>,
        population_size: usize,
        extra_population_size: usize,
    ) -> Self {
        let rng = RefCell::new(StdRng::seed_from_u64(42));
        Self {
            problem,
            population_size,
            extra_population_size,
            rng,
        }
    }
    pub fn crossover(
        parent1: &Solution,
        parent2: &Solution,
        start_index: usize,
        end_index: usize,
    ) -> Solution {
        let mut parent2_replaced_elements: BTreeSet<u32> = parent2.order[start_index..end_index]
            .iter()
            .cloned()
            .collect();
        let mut child_order = parent2.order.clone();
        for i in start_index..end_index {
            parent2_replaced_elements.remove(&parent1.order[i]);
            child_order[i] = parent1.order[i];
        }
        let mut visited = vec![false; parent1.order.len()];
        let mut dirty_indices = Vec::new();
        for i in 0..parent1.order.len() {
            if visited[child_order[i] as usize] {
                dirty_indices.push(i);
            } else {
                visited[child_order[i] as usize] = true;
            }
        }
        assert!(parent2_replaced_elements.len() == dirty_indices.len());
        for (dirty_index, replacement) in dirty_indices
            .into_iter()
            .zip(parent2_replaced_elements.into_iter())
        {
            child_order[dirty_index] = replacement;
        }
        let child = Solution::new(
            child_order,
            Rc::downgrade(&parent1.problem.upgrade().unwrap()),
            None,
        );
        assert!(child.is_valid(parent1.order.len() as u32));
        child
    }
    pub fn solve(&self, timer: &Instant, time_limit_secs: u64) -> Solution {
        let mut rng = self.rng.borrow_mut();
        let mut current_best_solution: Option<Rc<Solution>> = None;
        let mut current_best_distance = f32::INFINITY;
        let mut population: Vec<Rc<Solution>> = Vec::new();
        for i in 0..self.population_size {
            if timer.elapsed().as_secs() >= time_limit_secs {
                return current_best_solution
                    .as_ref()
                    .expect("No solution found")
                    .as_ref()
                    .clone();
            }
            let start_index = if i < self.problem.cities.len() {
                i
            } else {
                break;
            };
            let solution = Solution::from_nearest_neighbor(&self.problem, start_index);
            let solution = Rc::new(solution);
            let total_distance = solution.total_distance();
            if total_distance < current_best_distance {
                current_best_distance = total_distance;
                current_best_solution = Some(solution.clone());
                println!(
                    "New best solution found by nearest neighbor with start index {}: {}",
                    start_index, total_distance
                );
            }
            population.push(solution);
        }
        let mut population = Population::new(population);

        // for _i in 0..self.n_generations {
        loop {
            if timer.elapsed().as_secs() >= time_limit_secs {
                return current_best_solution
                    .as_ref()
                    .expect("No solution found")
                    .as_ref()
                    .clone();
            }
            let mut new_solutions = Vec::new();
            for _ in 0..self.extra_population_size {
                let parent1 = population.sample_parent(&mut rng);
                let parent2 = population.sample_parent(&mut rng);
                let start_index = rng.random_range(0..parent1.order.len());
                let end_index = rng.random_range(start_index..parent1.order.len());
                let child = Self::crossover(&parent1, &parent2, start_index, end_index);
                let child_distance = child.total_distance();
                if child_distance < current_best_distance {
                    current_best_distance = child_distance;
                    current_best_solution = Some(Rc::new(child.clone()));
                    println!("New best solution found by crossover: {}", child_distance);
                }
                new_solutions.push(Rc::new(child));
            }
            let mut all_solutions = population.solutions;
            all_solutions.extend(new_solutions);
            // extend the population
            population = Population::new(all_solutions);
            let mut new_solutions: BTreeSet<RcKey<Solution>> = BTreeSet::new();
            let mut num_trials = 0;
            while new_solutions.len() < self.population_size {
                if num_trials > self.population_size * 2 {
                    println!("Warning: Too many trials to sample new solutions, breaking");
                    break;
                }
                num_trials += 1;
                let solution = population.sample_parent(&mut rng);
                let solution_id = RcKey::new(solution.clone());
                if !new_solutions.contains(&solution_id) {
                    new_solutions.insert(solution_id);
                }
            }
            let new_solutions: Vec<Rc<Solution>> =
                new_solutions.into_iter().map(|k| k.rc().clone()).collect();
            population = Population::new(new_solutions);
        }
        // current_best_solution
        //     .as_ref()
        //     .expect("No solution found")
        //     .as_ref()
        //     .clone()
    }
}
