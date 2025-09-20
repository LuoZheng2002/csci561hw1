// use std::cmp::Ordering;
// use std::f64::INFINITY;

// use crate::problem::City;



// #[derive(Debug)]
// pub struct KDNode {
//     city: City,
//     left: Option<Box<KDNode>>,
//     right: Option<Box<KDNode>>,
//     axis: usize,
// }

// impl KDNode {
//     fn build(cities: &[City], depth: usize) -> Option<Box<KDNode>> {
//         if cities.is_empty() {
//             return None;
//         }

//         let axis = depth % 3;
//         let mut cities = cities.to_vec();
//         cities.sort_by(|a, b| {
//             a.coord(axis)
//                 .partial_cmp(&b.coord(axis))
//                 .unwrap_or(Ordering::Equal)
//         });

//         let mid = cities.len() / 2;

//         Some(Box::new(KDNode {
//             city: cities[mid],
//             axis,
//             left: KDNode::build(&cities[..mid], depth + 1),
//             right: KDNode::build(&cities[mid + 1..], depth + 1),
//         }))
//     }

//     fn nearest<'a>(
//         &'a self,
//         query: &City,
//         best: Option<(&'a City, f32)>,
//     ) -> (&'a City, f32) {
//         let mut best = match best {
//             Some((pt, dist)) => {
//                 let d = self.city.dist2(query);
//                 if d < dist {
//                     (&self.city, d)
//                 } else {
//                     (pt, dist)
//                 }
//             }
//             None => (&self.city, self.city.dist2(query)),
//         };

//         let axis = self.axis;
//         let go_left = query.coord(axis) < self.city.coord(axis);

//         let (first, second) = if go_left {
//             (&self.left, &self.right)
//         } else {
//             (&self.right, &self.left)
//         };

//         if let Some(child) = first {
//             best = child.nearest(query, Some(best));
//         }

//         // Check if we need to explore the other side
//         let plane_dist = (query.coord(axis) - self.city.coord(axis)).powi(2);
//         if plane_dist < best.1 {
//             if let Some(child) = second {
//                 best = child.nearest(query, Some(best));
//             }
//         }

//         best
//     }
// }

// pub struct KDTree {
//     root: Option<Box<KDNode>>,
// }

// impl KDTree {
//     pub fn new(cities: &[City]) -> Self {
//         KDTree {
//             root: KDNode::build(cities, 0),
//         }
//     }

//     pub fn nearest(&self, query: &City) -> Option<(City, f32)> {
//         self.root
//             .as_ref()
//             .map(|node| {
//                 let (pt, d2) = node.nearest(query, None);
//                 (*pt, d2.sqrt())
//             })
//     }
// }
