use core::f32;
use std::{
    cell::RefCell,
    cmp::{Ordering, Reverse},
    collections::{BTreeSet, BinaryHeap, VecDeque},
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

enum CrossoverState {
    FindingStartIndex,
    FindingDifferentAfterStart {
        start_index: u32,
    },
    FindingEndIndex {
        start_index: u32,
        parent1_set: BTreeSet<u32>,
        parent2_set: BTreeSet<u32>,
    },
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
    pub fn random_crossover(
        parent1: &Solution,
        parent2: &Solution,
        start_index: usize,
        end_index: usize,
    ) -> Solution {
        let mut parent2_replaced_elements: BTreeSet<u32> = parent2.order_without_loop
            [start_index..end_index]
            .iter()
            .cloned()
            .collect();
        let mut child_order = parent2.order_without_loop.clone();
        for i in start_index..end_index {
            parent2_replaced_elements.remove(&parent1.order_without_loop[i]);
            child_order[i] = parent1.order_without_loop[i];
        }
        let mut visited = vec![false; parent1.order_without_loop.len()];
        let mut dirty_indices = Vec::new();
        for i in 0..parent1.order_without_loop.len() {
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
        assert!(child.is_valid(parent1.order_without_loop.len() as u32));
        child
    }

    pub fn precise_crossover(
        parent1: &Solution,
        parent2: &Solution,
        callback: &mut dyn FnMut(&Solution) -> bool,
    ) {
        let parent1_order = &parent1.order_without_loop;
        let mut parent2_order_forward: VecDeque<u32> = parent2.order_without_loop.clone().into();
        let mut parent2_order_reversed: VecDeque<u32> =
            parent2.order_without_loop.iter().cloned().rev().collect();
        for _ in 0..parent1_order.len() {
            for parent2_order in [&mut parent2_order_forward, &mut parent2_order_reversed] {
                let mut next_start_to_explore: Option<u32> = Some(0);
                while let Some(current_start) = next_start_to_explore.take() {
                    let mut crossover_state = CrossoverState::FindingStartIndex;
                    for i in current_start as usize..parent1_order.len() {
                        let parent1_element = parent1_order[i];
                        let parent2_element = parent2_order[i];
                        match crossover_state {
                            CrossoverState::FindingStartIndex => {
                                if parent1_element == parent2_element {
                                    crossover_state = CrossoverState::FindingDifferentAfterStart {
                                        start_index: i as u32,
                                    };
                                }
                            }
                            CrossoverState::FindingDifferentAfterStart { start_index } => {
                                if parent1_element == parent2_element {
                                    assert!(i as u32 == start_index + 1);
                                    crossover_state = CrossoverState::FindingDifferentAfterStart {
                                        start_index: i as u32,
                                    };
                                } else {
                                    let mut parent1_set: BTreeSet<u32> = BTreeSet::new();
                                    let mut parent2_set: BTreeSet<u32> = BTreeSet::new();
                                    parent1_set.insert(parent1_element);
                                    parent2_set.insert(parent2_element);
                                    crossover_state = CrossoverState::FindingEndIndex {
                                        start_index,
                                        parent1_set,
                                        parent2_set,
                                    };
                                }
                            }
                            CrossoverState::FindingEndIndex {
                                start_index,
                                mut parent1_set,
                                mut parent2_set,
                            } => {
                                if parent1_element == parent2_element {
                                    if parent1_set == parent2_set {
                                        // println!(
                                        //     "Found a crossover between indices {} and {}",
                                        //     start_index, i
                                        // );
                                        // println!("Parent 1: {:?}", parent1_order);
                                        // println!("Parent 2: {:?}", parent2_order);
                                        let end_index = i;
                                        let mut new_child1_order = parent1_order.clone();
                                        let parent2_order_slice = (start_index as usize..end_index)
                                            .map(|x| *parent2_order.get(x as usize).unwrap())
                                            .collect::<Vec<u32>>();
                                        new_child1_order[start_index as usize..end_index]
                                            .copy_from_slice(parent2_order_slice.as_slice());
                                        let new_child1 = Solution::new(
                                            new_child1_order,
                                            Rc::downgrade(&parent1.problem.upgrade().unwrap()),
                                            None,
                                        );
                                        assert!(
                                            new_child1
                                                .is_valid(parent1.order_without_loop.len() as u32)
                                        );
                                        // let child = Self::crossover(parent1, parent2, start_index as usize, end_index);
                                        if !callback(&new_child1) {
                                            return;
                                        }
                                        let mut new_child2_order =
                                            parent2_order.iter().cloned().collect::<Vec<u32>>();
                                        new_child2_order[start_index as usize..end_index]
                                            .copy_from_slice(
                                                &parent1_order[start_index as usize..end_index],
                                            );
                                        let new_child2 = Solution::new(
                                            new_child2_order,
                                            Rc::downgrade(&parent2.problem.upgrade().unwrap()),
                                            None,
                                        );
                                        if !callback(&new_child2) {
                                            return;
                                        }
                                        return;
                                        assert!(
                                            new_child2
                                                .is_valid(parent2.order_without_loop.len() as u32)
                                        );
                                        crossover_state =
                                            CrossoverState::FindingDifferentAfterStart {
                                                start_index: i as u32,
                                            };
                                    } else {
                                        if next_start_to_explore.is_none() {
                                            next_start_to_explore = Some(i as u32);
                                        }
                                        parent1_set.insert(parent1_element);
                                        parent2_set.insert(parent2_element);
                                        crossover_state = CrossoverState::FindingEndIndex {
                                            start_index,
                                            parent1_set,
                                            parent2_set,
                                        };
                                    }
                                } else {
                                    parent1_set.insert(parent1_element);
                                    parent2_set.insert(parent2_element);
                                    crossover_state = CrossoverState::FindingEndIndex {
                                        start_index,
                                        parent1_set,
                                        parent2_set,
                                    };
                                }
                            }
                        }
                    }
                }
                parent2_order.rotate_right(1);
            }
        }
    }

    pub fn solve(&self, timer: &Instant, time_limit_secs: u64) -> Solution {
        let mut rng = self.rng.borrow_mut();
        let mut current_best_solution: Option<Rc<Solution>> = None;
        let mut current_best_distance = f32::INFINITY;
        let mut population: Vec<Rc<Solution>> = Vec::new();
        // let mut population: BinaryHeap<Reverse<(NotNan<f32>, RcKey<Solution>)>> = BinaryHeap::new();

        let num_cities = self.problem.cities.len();

        let mut visited_total_lengths: BTreeSet<NotNan<f32>> = BTreeSet::new();

        // the initial population are all generated from nearest neighbor with different starting points
        for start_index in 0..num_cities {
            if timer.elapsed().as_secs() >= time_limit_secs {
                return current_best_solution
                    .as_ref()
                    .expect("No solution found")
                    .as_ref()
                    .clone();
            }
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
            visited_total_lengths.insert(NotNan::new(total_distance).unwrap());
            population.push(solution);
        }
        // let population: Vec<Rc<Solution>> =
        //     population.drain().map(|rev| rev.0.1.rc().clone()).collect();
        let mut population = Population::new(population);
        assert!(population.solutions.len() > 1);
        let mut generation: u32 = 0;
        loop {
            println!("Generation {}", generation);
            generation += 1;
            if timer.elapsed().as_secs() >= time_limit_secs {
                return current_best_solution
                    .as_ref()
                    .expect("No solution found")
                    .as_ref()
                    .clone();
            }
            let mut new_solutions = Vec::new();
            let total_crossover_trials = self.extra_population_size * 2;
            // let precise_crossover_trials = self.extra_population_size * 2;
            for _ in 0..total_crossover_trials {
                if population.solutions.len() + new_solutions.len()
                    >= self.population_size + self.extra_population_size
                {
                    println!(
                        "Population full, stopping all crossover -----------------------------------"
                    );
                    break;
                }

                const PRECISE_CROSSOVERS_PER_TRIAL: usize = 4;
                for _ in 0..PRECISE_CROSSOVERS_PER_TRIAL {
                    if timer.elapsed().as_secs() >= time_limit_secs {
                        println!(
                            "Time limit reached, stopping all crossover -----------------------------------"
                        );
                        break;
                    }
                    if population.solutions.len() + new_solutions.len()
                        >= self.population_size + self.extra_population_size
                    {
                        println!(
                            "Population full, stopping all crossover -----------------------------------"
                        );
                        break;
                    }
                    let (parent1, parent2) = loop {
                        let parent1 = population.sample_parent(&mut rng);
                        let parent2 = population.sample_parent(&mut rng);
                        if !Rc::ptr_eq(&parent1, &parent2) {
                            break (parent1, parent2);
                        }
                    };
                    assert!(parent1.is_valid(num_cities as u32));
                    // Self::crossover(parent1, parent2, start_index, end_index)
                    let mut callback = |child: &Solution| -> bool {
                        assert!(
                            population.solutions.len() + new_solutions.len()
                                < self.population_size + self.extra_population_size
                        );
                        let child_distance = child.total_distance();
                        if child_distance < current_best_distance {
                            current_best_distance = child_distance;
                            current_best_solution = Some(Rc::new(child.clone()));
                            println!(
                                "New best solution found by precise crossover: {}",
                                child_distance
                            );
                        }
                        if visited_total_lengths.insert(NotNan::new(child_distance).unwrap()) {
                            // println!(
                            //     "new precise crossover distance: {}, new solution size: {}",
                            //     child_distance,
                            //     new_solutions.len()
                            // );
                            new_solutions.push(Rc::new(child.clone()));
                        }
                        let population_not_full = population.solutions.len() + new_solutions.len()
                            < self.population_size + self.extra_population_size;
                        let has_time = timer.elapsed().as_secs() < time_limit_secs;
                        // returns true if we want to continue generating more children
                        population_not_full && has_time
                    };
                    Self::precise_crossover(&parent1, &parent2, &mut callback);
                }
                const RANDOM_CROSSOVER_PER_TRIAL: usize = 1;
                for _ in 0..RANDOM_CROSSOVER_PER_TRIAL {
                    if timer.elapsed().as_secs() >= time_limit_secs {
                        println!(
                            "Time limit reached, stopping all crossover -----------------------------------"
                        );
                        break;
                    }
                    let (parent1, parent2) = loop {
                        let parent1 = population.sample_parent(&mut rng);
                        let parent2 = population.sample_parent(&mut rng);
                        if !Rc::ptr_eq(&parent1, &parent2) {
                            break (parent1, parent2);
                        }
                    };
                    assert!(parent1.is_valid(num_cities as u32));
                    let (start_index, end_index) = loop {
                        let index1 = rng.random_range(0..num_cities);
                        let index2 = rng.random_range(0..num_cities);
                        if index1 < index2 {
                            break (index1, index2);
                        } else if index1 > index2 {
                            break (index2, index1);
                        }
                    };
                    let child = Self::random_crossover(&parent1, &parent2, start_index, end_index);
                    let child_distance = child.total_distance();
                    if child_distance < current_best_distance {
                        current_best_distance = child_distance;
                        current_best_solution = Some(Rc::new(child.clone()));
                        println!(
                            "New best solution found by random crossover: {}",
                            child_distance
                        );
                    }
                    if visited_total_lengths.insert(NotNan::new(child_distance).unwrap()) {
                        // println!(
                        //     "new random crossover distance: {}, new solution size: {}",
                        //     child_distance,
                        //     new_solutions.len()
                        // );
                        new_solutions.push(Rc::new(child));
                    }
                }
                if timer.elapsed().as_secs() >= time_limit_secs {
                    println!(
                        "Time limit reached, exiting from crossover -----------------------------------"
                    );
                    break;
                }
            }
            if timer.elapsed().as_secs() >= time_limit_secs {
                return current_best_solution
                    .as_ref()
                    .expect("No solution found")
                    .as_ref()
                    .clone();
            }
            let mut combined_solutions = population.solutions;
            combined_solutions.extend(new_solutions);
            let mut combined_population = Population::new(combined_solutions);
            // let mut new_solutions: BTreeSet<RcKey<Solution>> = BTreeSet::new();
            let new_solutions =
                combined_population.shrink_population(self.population_size, &mut rng);
            population = Population::new(new_solutions);
            // while new_solutions.len() < self.population_size {
            //     if num_trials > self.population_size * 3 / 2 {
            //         println!(
            //             "Warning: Too many trials to sample new solutions. Collected {} unique solutions.",
            //             new_solutions.len()
            //         );
            //         break;
            //     }
            //     num_trials += 1;
            //     let solution = population.sample_parent(&mut rng);
            //     let solution_id = RcKey::new(solution.clone());
            //     if !new_solutions.contains(&solution_id) {
            //         new_solutions.insert(solution_id);
            //     }
            // }
            // let new_solutions: Vec<Rc<Solution>> =
            //     new_solutions.into_iter().map(|k| k.rc().clone()).collect();
            // population = Population::new(new_solutions);
        }
        // current_best_solution
        //     .as_ref()
        //     .expect("No solution found")
        //     .as_ref()
        //     .clone()
    }
}
