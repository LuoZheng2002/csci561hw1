use core::f32;
use std::{cell::RefCell, rc::{Rc, Weak}};

use ordered_float::NotNan;
use rand::{distr::{weighted::WeightedIndex, Distribution}, seq::SliceRandom, Rng};

use crate::{cover_tree::CoverTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct City{
    pub x: NotNan<f32>,
    pub y: NotNan<f32>,
    pub z: NotNan<f32>,
}

impl City{
    pub fn dist2(&self, other: &Self) -> f32 {
        (self.x - other.x).powi(2) +
        (self.y - other.y).powi(2) +
        (self.z - other.z).powi(2)
    }
    pub fn distance(&self, other: &Self) -> f32 {
        self.dist2(other).sqrt()
    }
}

pub struct Problem{
    pub cities: Vec<City>,
}

#[derive(Clone)]
pub struct Solution{
    pub order: Vec<usize>,
    pub problem: Weak<Problem>,
    total_length: RefCell<Option<f32>>,
    pub id: usize,
}

impl Solution {
    pub fn new(order: Vec<usize>, 
        problem: Weak<Problem>, 
        id_iter: &mut dyn Iterator<Item = usize>
    ) -> Self {
        Self {
            order,
            problem,
            total_length: RefCell::new(None),
            id: id_iter.next().unwrap(),
        }
    }
    pub fn is_valid(&self, num_cities: usize) -> bool {
        if self.order.len() != num_cities {
            return false;
        }
        // check for duplicates
        let mut seen = vec![false; num_cities];
        for &city in &self.order {
            if city >= num_cities || seen[city] {
                return false;
            }
            seen[city] = true;
        }
        true
    }
    pub fn total_length(&self) -> f32 {
        let mut total_length = self.total_length.borrow_mut();
        if let Some(length) = total_length.as_ref() {
            return *length;
        }
        let mut new_total_length = 0.0;
        let problem = self.problem.upgrade().expect("Problem has been dropped");
        for i in 0..self.order.len() {
            let city_a = &problem.cities[self.order[i]];
            let city_b = &problem.cities[self.order[(i + 1) % self.order.len()]];
            let dist = ((city_a.x - city_b.x).powi(2)
                + (city_a.y - city_b.y).powi(2)
                + (city_a.z - city_b.z).powi(2))
                .sqrt();
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
    pub fn from_random_shuffle(problem: &Rc<Problem>, rng: &mut impl Rng, id_iter: &mut dyn Iterator<Item = usize>) -> Self {
        let num_cities = problem.cities.len();
        let mut order: Vec<usize> = (0..num_cities).collect();
        order.shuffle(rng);
        Self::new(order, Rc::downgrade(problem), id_iter)
    }

    pub fn from_nearest_neighbor(problem: &Rc<Problem>){
        // let cover_tree = CoverTree::new()
    }
}

pub struct Population{
    pub solutions: Vec<Rc<Solution>>,
    max_length: RefCell<Option<f32>>,
    min_length: RefCell<Option<f32>>,
    roulette: RefCell<Option<WeightedIndex<f32>>>,
}

impl Population{
    pub fn new(solutions: Vec<Rc<Solution>>) -> Self {
        Self {
            solutions,
            max_length: RefCell::new(None),
            min_length: RefCell::new(None),
            roulette: RefCell::new(None),
        }
    }
    fn get_min_max(&self) -> (f32, f32){
        let mut min_length = f32::MAX;
        let mut max_length = f32::MIN;
        for solution in &self.solutions {
            let length = solution.total_length();
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
        // let (min_length, max_length) = {
        //     let mut min_length = self.min_length.borrow_mut();
        //     let mut max_length = self.max_length.borrow_mut();
        //     match (min_length.as_ref(), max_length.as_ref()) {
        //         (Some(min_length), Some(max_length)) => (*min_length, *max_length),
        //         _ => {
        //             let (new_min_length, new_max_length) = self.get_min_max();
        //             *min_length = Some(new_min_length);
        //             *max_length = Some(new_max_length);
        //             (new_min_length, new_max_length)
        //         }
        //     }
        // };
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
        let fitnesses = self.solutions.iter().map(|solution| {
            let length = solution.total_length();
            Self::calculate_fitness(length, min_length, max_length)
        }).collect::<Vec<_>>();
        // weighted index will normalize the fitnesses for us
        // println!("Fitnesses: {:?}", fitnesses);
        WeightedIndex::new(fitnesses).expect("Failed to create WeightedIndex")
    }
    pub fn from_random_shuffle(problem: &Rc<Problem>, size: usize, rng: &mut impl Rng, id_iter: &mut dyn Iterator<Item = usize>) -> Self {
        let solutions = (0..size)
            .map(|_| {
                let solution = Solution::from_random_shuffle(problem, rng, id_iter);
                println!("From random shuffle: id: {}, total_length: {}", solution.id, solution.total_length());
                Rc::new(solution)
            })
            .collect();
        Population::new(solutions)
    }
    pub fn sample_parent(&self, rng: &mut impl Rng) -> Rc<Solution> {
        let mut roulette = self.roulette.borrow_mut();
        let roulette = roulette.get_or_insert_with(|| self.calculate_roulette());
        let index = roulette.sample(rng);
        self.solutions[index].clone()
    }
    pub fn best_solution(&self) -> Rc<Solution> {
        self.solutions.iter().min_by(|a, b| {
            a.total_length()
                .partial_cmp(&b.total_length())
                .unwrap_or(std::cmp::Ordering::Equal)
        }).expect("Population has no solutions").clone()
    }
}