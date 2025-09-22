use core::f32;
use std::{
    cell::RefCell,
    cmp::{Ordering, Reverse},
    collections::{BTreeSet, BinaryHeap},
    rc::Rc,
    time::Instant,
};

use ordered_float::NotNan;
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

use crate::problem::{Population, Problem, RcKey, Solution};

pub struct GeneticAlgorithm {
    pub problem: Rc<Problem>,
    pub population_size: usize,
    pub extra_population_size: usize,
    rng: RefCell<StdRng>,
}

impl GeneticAlgorithm {
    pub fn new(problem: Rc<Problem>, population_size: usize, extra_population_size: usize) -> Self {
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
        // let mut population: Vec<Rc<Solution>> = Vec::new();
        let mut population: BinaryHeap<Reverse<(NotNan<f32>, RcKey<Solution>)>> = BinaryHeap::new();
        let num_cities = self.problem.cities.len();
        let first_proba_threshold = num_cities;
        let second_proba_threshold = num_cities * 2;

        let mut visited_total_lengths: BTreeSet<NotNan<f32>> = BTreeSet::new();

        let mut nn_start_index: usize = 0;
        let mut num_nearest_neighbor_calls = 0;
        let mut execute_nearest_neighbor = |nn_start_index: &mut usize, rng: &mut StdRng| {
            // let start_index = i % num_cities;
            let second_nearest_proba = if num_nearest_neighbor_calls < first_proba_threshold {
                0.5
            } else if num_nearest_neighbor_calls >= first_proba_threshold
                && num_nearest_neighbor_calls < second_proba_threshold
            {
                0.5 * (num_nearest_neighbor_calls - first_proba_threshold) as f32
                    / (second_proba_threshold - first_proba_threshold) as f32
            } else {
                0.5
            };
            if num_nearest_neighbor_calls == second_proba_threshold {
                println!("Second proba threshold reached");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            let nth_neighbor = if rng.random_bool(second_nearest_proba as f64) {
                2
            } else {
                1
            };
            let solution =
                Solution::from_nearest_neighbor(&self.problem, *nn_start_index, nth_neighbor);
            num_nearest_neighbor_calls += 1;
            *nn_start_index = (*nn_start_index + 1) % num_cities;
            solution
        };

        // for i in 0..self.population_size {
        loop {
            if timer.elapsed().as_secs() >= (time_limit_secs + 1) / 2 {
                // return current_best_solution
                //     .as_ref()
                //     .expect("No solution found")
                //     .as_ref()
                //     .clone();
                println!("Half time reached, break population initialization");
                break;
            }
            // let start_index = if i < num_cities {
            //     i
            // } else {
            //     break;
            // };
            let solution = execute_nearest_neighbor(&mut nn_start_index, &mut rng);
            let solution = Rc::new(solution);
            let total_distance = solution.total_distance();
            if total_distance < current_best_distance {
                current_best_distance = total_distance;
                current_best_solution = Some(solution.clone());
                println!(
                    "New best solution found by nearest neighbor with start index {}: {}",
                    nn_start_index, total_distance
                );
            }
            // population.push(solution);
            if visited_total_lengths.insert(NotNan::new(total_distance).unwrap()) {
                // println!(
                //     "found a suboptimal solution from nn with length {}",
                //     total_distance
                // );
                // only insert if this total distance has not been seen before
                population.push(Reverse((
                    NotNan::new(total_distance).unwrap(),
                    RcKey::new(solution.clone()),
                )));
                if population.len() > self.population_size {
                    population.pop();
                }
            }
        }
        let population: Vec<Rc<Solution>> =
            population.drain().map(|rev| rev.0.1.rc().clone()).collect();
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
            for _ in 0..self.extra_population_size / 2 {
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
                if visited_total_lengths.insert(NotNan::new(child_distance).unwrap()) {
                    // println!(
                    //     "found a suboptimal solution from crossover with length {}",
                    //     child_distance
                    // );
                    // only insert if this total distance has not been seen before
                    // population.push(Rc::new(child.clone()));
                    new_solutions.push(Rc::new(child.clone()));
                }
            }
            for _ in 0..self.extra_population_size / 2 {
                if timer.elapsed().as_secs() >= time_limit_secs {
                    return current_best_solution
                        .as_ref()
                        .expect("No solution found")
                        .as_ref()
                        .clone();
                }
                let solution = execute_nearest_neighbor(&mut nn_start_index, &mut rng);
                let solution = Rc::new(solution);
                let total_distance = solution.total_distance();
                if total_distance < current_best_distance {
                    current_best_distance = total_distance;
                    current_best_solution = Some(solution.clone());
                    println!(
                        "New best solution found by nearest neighbor with start index {}: {}",
                        nn_start_index, total_distance
                    );
                }
                // population.push(solution);
                if visited_total_lengths.insert(NotNan::new(total_distance).unwrap()) {
                    // println!(
                    //     "found a suboptimal solution from nn with length {}",
                    //     total_distance
                    // );
                    // only insert if this total distance has not been seen before
                    new_solutions.push(solution.clone());
                }
            }
            let mut all_solutions = population.solutions;
            all_solutions.extend(new_solutions);
            // extend the population
            population = Population::new(all_solutions);
            let mut new_solutions: BTreeSet<RcKey<Solution>> = BTreeSet::new();
            let mut num_trials = 0;
            while new_solutions.len() < self.population_size {
                if num_trials > self.population_size * 3 / 2 {
                    println!(
                        "Warning: Too many trials to sample new solutions. Collected {} unique solutions.",
                        new_solutions.len()
                    );
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
