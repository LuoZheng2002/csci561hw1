use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

use crate::problem::{Population, Problem, Solution};

pub struct GeneticAlgorithm {
    pub problem: Rc<Problem>,
    pub population_size: usize, 
    pub n_generations: usize, 
    pub extra_population_size: usize,
    rng: RefCell<StdRng>,
}


impl GeneticAlgorithm{
    pub fn new(problem: Rc<Problem>, population_size: usize, n_generations: usize, extra_population_size: usize) -> Self {
        let rng = RefCell::new(StdRng::seed_from_u64(42));
        Self {
            problem,
            population_size,
            n_generations,
            extra_population_size,
            rng,
        }
    }
    pub fn crossover(
        parent1: &Solution, 
        parent2: &Solution, 
        start_index: usize,
        end_index: usize,
        id_iter: &mut dyn Iterator<Item = usize>,
    ) -> Solution {
        let mut parent2_replaced_elements: BTreeSet<usize> = parent2.order[start_index..end_index]
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
        for i in 0..parent1.order.len(){
            if visited[child_order[i]] {
                dirty_indices.push(i);
            } else {
                visited[child_order[i]] = true;
            }
        }
        assert!(parent2_replaced_elements.len() == dirty_indices.len());
        for (dirty_index, replacement) in dirty_indices.into_iter().zip(parent2_replaced_elements.into_iter()) {
            child_order[dirty_index] = replacement;
        }
        let child = Solution::new(child_order, Rc::downgrade(&parent1.problem.upgrade().unwrap()), id_iter);
        assert!(child.is_valid(parent1.order.len()));
        child
    }
    pub fn solve(&self) -> Rc<Solution> {
        let mut rng = self.rng.borrow_mut();
        let mut id_iter = 0..;
        let mut population = Population::from_random_shuffle(&self.problem, self.population_size, &mut rng, &mut id_iter);
        for i in 0..self.n_generations {
            println!("Generation {}", i);
            let mut new_solutions = Vec::new(); 
            for _ in 0..self.extra_population_size{                
                let parent1 = population.sample_parent(&mut rng);
                let parent2 = population.sample_parent(&mut rng);
                let start_index = rng.random_range(0..parent1.order.len());
                let end_index = rng.random_range(start_index..parent1.order.len());
                let child = Self::crossover(&parent1, &parent2, start_index, end_index, &mut id_iter);
                // println!("Crossover: id: {}, total_length: {}", child.id, child.total_length());
                new_solutions.push(Rc::new(child));
            }
            let mut all_solutions = population.solutions;
            all_solutions.extend(new_solutions);
            // extend the population
            population = Population::new(all_solutions);
            let mut new_solution_ids = BTreeSet::new();
            let mut new_solutions = Vec::new();
            while new_solutions.len() < self.population_size {
                let solution = population.sample_parent(&mut rng);
                let solution_id = solution.id;
                if !new_solution_ids.contains(&solution_id) {
                    new_solution_ids.insert(solution_id);
                    new_solutions.push(solution);
                    // println!("Successfully added solution id: {}, solution num: {}", solution_id, new_solutions.len());
                }else{
                    // println!("Duplicate solution id found: {}", solution_id);
                    // std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            // shrink the population back to the original size
            population = Population::new(new_solutions);
        }
        population.best_solution()
    }
}