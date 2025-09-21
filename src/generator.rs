use ordered_float::NotNan;
use rand::distr::Uniform;
use rand::prelude::*;
use std::f64::consts::PI;
use std::{cell::RefCell, collections::BTreeSet};

use crate::problem::{City, Problem};

pub struct ProblemGenerator {
    pub num_cities: usize,
    pub max_radius: f32,
    pub rng: RefCell<StdRng>,
}

fn generate_point_in_sphere(max_radius: f32, dist: &Uniform<f32>, rng: &mut impl Rng) -> City {
    let u = rng.sample(dist);
    let v = rng.sample(dist);
    let w = rng.sample(dist);

    let theta = 2.0 * PI as f32 * u;
    let phi = (2.0 * v - 1.0).acos();
    let r = w.cbrt() * max_radius;

    let x = r * phi.sin() * theta.cos();
    let y = r * phi.sin() * theta.sin();
    let z = r * phi.cos();

    let x = x as u32;
    let y = y as u32;
    let z = z as u32;

    City { x, y, z }
}
impl ProblemGenerator {
    pub fn new(num_cities: usize, max_radius: f32, seed: u64) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        Self {
            num_cities,
            max_radius,
            rng: RefCell::new(rng),
        }
    }
    pub fn generate_problem(&self) -> Problem {
        let mut rng = self.rng.borrow_mut();
        let dist = Uniform::new(0.0, 1.0).unwrap();
        let mut cities: BTreeSet<City> = BTreeSet::new();
        while cities.len() < self.num_cities {
            let city = generate_point_in_sphere(self.max_radius, &dist, &mut *rng);
            cities.insert(city);
        }
        let mut cities = cities.into_iter().collect::<Vec<_>>();
        cities.shuffle(&mut *rng);
        Problem { cities }
    }
}
