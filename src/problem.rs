use core::f32;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::BinaryHeap,
    rc::{Rc, Weak},
};

use itertools::Itertools;
use ordered_float::NotNan;
use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
    seq::SliceRandom,
};

use crate::cover_tree::{CoverTree, Distance};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct City {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl City {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone)]
pub struct RcKey<T>(Rc<T>);

impl<T> RcKey<T> {
    pub fn new(inner: Rc<T>) -> Self {
        RcKey(inner)
    }
    pub fn rc(&self) -> &Rc<T> {
        &self.0
    }
}

impl<T> PartialEq for RcKey<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for RcKey<T> {}

impl<T> PartialOrd for RcKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for RcKey<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare the raw pointer addresses
        // self.0.as_ptr().cmp(&other.0.as_ptr())
        Rc::as_ptr(&self.0).cmp(&Rc::as_ptr(&other.0))
    }
}

// impl City {
//     pub fn dist2(&self, other: &Self) -> f32 {
//         (self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)
//     }
// }

impl Distance for City {
    fn distance(&self, other: &Self) -> f32 {
        let dx = self.x as i32 - other.x as i32;
        let dy = self.y as i32 - other.y as i32;
        let dz = self.z as i32 - other.z as i32;
        ((dx * dx + dy * dy + dz * dz) as f32).sqrt()
    }
}
impl std::fmt::Display for City {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.0}, {:.0}, {:.0})", self.x, self.y, self.z)
    }
}

impl std::fmt::Debug for City {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.0}, {:.0}, {:.0})", self.x, self.y, self.z)
    }
}

pub struct Problem {
    pub cities: Vec<City>,
}

#[derive(Clone)]
pub struct Solution {
    pub order_without_loop: Vec<u32>,
    pub problem: Weak<Problem>,
    total_distance: RefCell<Option<f32>>,
}

impl Solution {
    pub fn new(
        order_without_loop: Vec<u32>,
        problem: Weak<Problem>,
        total_distance: Option<f32>,
    ) -> Self {
        Self {
            order_without_loop,
            problem,
            total_distance: RefCell::new(total_distance),
        }
    }
    pub fn is_valid(&self, num_cities: u32) -> bool {
        if self.order_without_loop.len() != num_cities as usize {
            println!(
                "Length mismatch: expected {}, got {}",
                num_cities,
                self.order_without_loop.len()
            );
            return false;
        }
        // check for duplicates
        let mut seen = vec![false; num_cities as usize];
        for &city in &self.order_without_loop {
            if city >= num_cities || seen[city as usize] {
                println!("Invalid city index or duplicate: {}", city);
                return false;
            }
            seen[city as usize] = true;
        }
        true
    }
    pub fn total_distance(&self) -> f32 {
        let mut total_length = self.total_distance.borrow_mut();
        if let Some(length) = total_length.as_ref() {
            return *length;
        }
        let mut new_total_length = 0.0;
        let problem = self.problem.upgrade().expect("Problem has been dropped");
        for i in 0..self.order_without_loop.len() {
            let city_a = &problem.cities[self.order_without_loop[i] as usize];
            let city_b = &problem.cities
                [self.order_without_loop[(i + 1) % self.order_without_loop.len()] as usize];
            let dist = city_a.distance(city_b);
            new_total_length += dist;
        }
        *total_length = Some(new_total_length);
        new_total_length
    }
    // pub fn fitness(&self, min_length: f32, max_length: f32) -> f32 {
    //     let total_length = self.total_length();
    //     let epsilon: f32 = 1e-6;
    //     let new_fitness = (max_length - length + epsilon) / (max_length - min_length + epsilon);
    //     *fitness = Some(new_fitness);
    //     new_fitness
    // }
    // pub fn from_random_shuffle(problem: &Rc<Problem>, rng: &mut impl Rng) -> Self {
    //     let num_cities = problem.cities.len();
    //     let mut order: Vec<u32> = (0..num_cities as u32).collect();
    //     order.shuffle(rng);
    //     Self::new(order, Rc::downgrade(problem), None)
    // }

    pub fn from_nearest_neighbor(problem: &Rc<Problem>, start_index: usize) -> Self {
        assert!(start_index < problem.cities.len());
        let mut cover_tree = CoverTree::new();
        let offset = start_index;
        let initial_city = problem.cities[offset];
        let mut current_city = initial_city;
        let mut total_distance = 0.0;
        let mut ordered_cities = vec![offset as u32];
        // populate the cover tree
        for (i, city) in problem.cities.iter().enumerate() {
            cover_tree.insert(*city, i as u32);
        }
        cover_tree.remove(&initial_city);
        for _ in 0..problem.cities.len() - 1 {
            let (nearest_city, index, distance) =
                cover_tree.nearest_neighbor(&current_city).unwrap();
            // println!("Current city: {:?}, Nearest city: {:?}, Dist: {}", current_city, nearest_city, dist);
            // current_city = nearest_city;
            total_distance += distance;
            cover_tree.remove(&nearest_city);
            ordered_cities.push(index);
            current_city = nearest_city;
        }
        assert!(cover_tree.is_empty());
        // ordered_cities.push(0);
        total_distance += current_city.distance(&initial_city);
        Self::new(ordered_cities, Rc::downgrade(problem), Some(total_distance))
    }
    // pub fn targeted_two_opt(&self) -> Self {
    //     let problem = self.problem.upgrade().expect("Problem has been dropped");
    //     let mut best_order = self.order_without_loop.clone();
    //     // let mut best_distance = self.total_distance();

    //     let n = best_order.len();
    //     if n < 4 {
    //         panic!("Solution too short for 2-opt optimization");
    //     }

    //     let cities = &problem.cities;

    //     // let dist = |a: u32, b: u32| cities[a as usize].distance(&cities[b as usize]);
    //     loop {
    //         let mut found_shorter = false;
    //         let start = best_order[0];
    //         let end = best_order[n - 1];
    //         for i in 1..n - 2 {
    //             let a = best_order[i];
    //             let b = best_order[i + 1];
    //             let city_a = &cities[a as usize];
    //             let city_b = &cities[b as usize];
    //             let city_start = &cities[start as usize];
    //             let city_end = &cities[end as usize];
    //             let current_cost = city_a.distance(city_b) + city_end.distance(city_start);
    //             let new_cost = city_a.distance(city_end) + city_b.distance(city_start);
    //             if new_cost < current_cost {
    //                 best_order[0..=i].reverse();
    //                 let new_solution =
    //                     Solution::new(best_order.clone(), Rc::downgrade(&problem), None);
    //                 let new_distance = new_solution.total_distance();
    //                 assert!(self.total_distance() - current_cost == new_distance - new_cost);
    //                 found_shorter = true;
    //                 println!("found shorter");
    //                 break;
    //             } else {
    //                 println!("not shorter");
    //             }
    //         }
    //         if !found_shorter {
    //             println!("did not find shorter");
    //             break;
    //         }
    //     }
    //     Solution::new(best_order, Rc::downgrade(&problem), None)
    // }

    pub fn from_brute_force(problem: &Rc<Problem>) -> Self {
        let cities = &problem.cities;
        if cities.len() <= 1 {
            return Solution::new(
                (0..cities.len() as u32).collect(),
                Rc::downgrade(problem),
                Some(0.0),
            );
        }
        // let start = cities[0];
        // let others: Vec<u32> = cities[1..].to_vec();
        let start_index = 0;
        let other_indices = (1..cities.len() as u32).collect::<Vec<_>>();

        let mut best_order = Vec::new();
        let mut best_distance = f32::INFINITY;

        for perm in other_indices
            .iter()
            .permutations(other_indices.len())
            .unique()
        {
            let mut candidate = vec![start_index];
            candidate.extend(perm.into_iter().copied());
            let mut total_distance = 0.0;
            for i in 0..candidate.len() {
                let city_a = &cities[candidate[i] as usize];
                let city_b = &cities[candidate[(i + 1) % candidate.len()] as usize];
                total_distance += city_a.distance(city_b);
            }
            if total_distance < best_distance {
                best_distance = total_distance;
                best_order = candidate;
            }
        }
        Solution::new(best_order, Rc::downgrade(&problem), Some(best_distance))
    }
}

pub struct Population {
    pub solutions: Vec<Rc<Solution>>,
    roulette: RefCell<Option<WeightedIndex<f32>>>,
    max_length: RefCell<Option<f32>>,
    min_length: RefCell<Option<f32>>,
}

impl Population {
    pub fn new(solutions: Vec<Rc<Solution>>) -> Self {
        Self {
            solutions,
            max_length: RefCell::new(None),
            min_length: RefCell::new(None),
            roulette: RefCell::new(None),
        }
    }
    fn get_min_max(&self) -> (f32, f32) {
        let mut min_length = f32::MAX;
        let mut max_length = f32::MIN;
        for solution in &self.solutions {
            let length = solution.total_distance();
            if length < min_length {
                min_length = length;
            }
            if length > max_length {
                max_length = length;
            }
        }
        (min_length, max_length)
    }
    fn calculate_fitness(total_length: f32, min_length: f32, max_length: f32) -> f32 {
        let epsilon: f32 = 1e-6;
        (max_length - total_length + epsilon) / (max_length - min_length + epsilon)
    }
    fn calculate_roulette(&self) -> WeightedIndex<f32> {
        let mut min_length = self.min_length.borrow_mut();
        let mut max_length = self.max_length.borrow_mut();
        if min_length.is_none() || max_length.is_none() {
            let (new_min_length, new_max_length) = self.get_min_max();
            *min_length = Some(new_min_length);
            *max_length = Some(new_max_length);
        }
        let min_length = min_length.unwrap();
        let max_length = max_length.unwrap();
        // println!("Min length: {}, Max length: {}", min_length, max_length);
        // calculate unnormalized fitnesses
        let fitnesses = self
            .solutions
            .iter()
            .map(|solution| {
                let length = solution.total_distance();
                Self::calculate_fitness(length, min_length, max_length)
            })
            .collect::<Vec<_>>();
        // weighted index will normalize the fitnesses for us
        // println!("Fitnesses: {:?}", fitnesses);
        WeightedIndex::new(fitnesses).expect("Failed to create WeightedIndex")
    }
    // pub fn from_random_shuffle(problem: &Rc<Problem>, size: usize, rng: &mut impl Rng) -> Self {
    //     let solutions = (0..size)
    //         .map(|_| {
    //             let solution = Solution::from_random_shuffle(problem, rng);
    //             Rc::new(solution)
    //         })
    //         .collect();
    //     Population::new(solutions)
    // }
    pub fn sample_parent(&self, rng: &mut impl Rng) -> Rc<Solution> {
        let mut roulette = self.roulette.borrow_mut();
        let roulette = roulette.get_or_insert_with(|| self.calculate_roulette());
        let index = roulette.sample(rng);
        self.solutions[index].clone()
    }
    pub fn shrink_population(&mut self, target_size: usize, rng: &mut StdRng) -> Vec<Rc<Solution>> {
        if self.solutions.len() <= target_size {
            // No need to shrink
            return self.solutions.clone();
        }
        // reset roulette and min/max lengths
        self.roulette.borrow_mut().take();
        self.min_length.borrow_mut().take();
        self.max_length.borrow_mut().take();

        let half_target_size = target_size / 2;
        let (first_half, _, _) = self
            .solutions
            .select_nth_unstable_by_key(half_target_size, |solution| {
                NotNan::new(solution.total_distance()).unwrap()
            });
        assert!(first_half.len() == half_target_size);
        let mut shrunk_population = first_half.to_vec();
        let mut last_half = self.solutions[half_target_size..]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let target_last_half_size = target_size - half_target_size;
        for _ in 0..target_last_half_size {
            let random_index = rng.random_range(0..last_half.len());
            let solution = last_half.swap_remove(random_index);
            shrunk_population.push(solution);
        }
        assert!(shrunk_population.len() == target_size);
        shrunk_population
    }
    // pub fn best_solution(&self) -> Rc<Solution> {
    //     self.solutions
    //         .iter()
    //         .min_by(|a, b| {
    //             a.total_distance()
    //                 .partial_cmp(&b.total_distance())
    //                 .unwrap_or(std::cmp::Ordering::Equal)
    //         })
    //         .expect("Population has no solutions")
    //         .clone()
    // }
}
