use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::{Rc, Weak};

use crate::problem::City;



#[derive(Debug)]
pub struct CoverTreeNode {
    city: City,
    level: RefCell<i32>,
    parent: RefCell<Weak<CoverTreeNode>>,
    children: RefCell<Vec<Rc<CoverTreeNode>>>,
}

impl CoverTreeNode {
    fn new(city: City, level: i32, parent: Weak<CoverTreeNode>) -> Rc<Self> {
        Rc::new(CoverTreeNode {
            city,
            level: RefCell::new(level),
            parent: RefCell::new(parent),
            children: RefCell::new(vec![]),
        })
    }

    /// Insert a new city into this node or its descendants
    /// Returns true if there is a parent found
    /// 
    fn insert(self: Rc<Self>, p: City) {
        self.children
            .borrow_mut()
            .push(CoverTreeNode::new(p, self.level.borrow().clone() - 1, Rc::downgrade(&self)));
    }
    
    // fn insert(self: Rc<Self>, p: City) -> bool {
    //     let self_level = self_node.level.borrow().clone();
    //     let mut candidates = vec![Rc::clone(self_node)];
    //     let mut level = self_level;

    //     loop {
    //         let mut next_candidates = vec![];
    //         let mut found = false;

    //         for candidate in &candidates {
    //             // let dist = candidate.borrow().city.distance(&p);
    //             let dist = candidate.city.distance(&p);
    //             if dist <= f32::exp2(level as f32) {
    //                 found = true;
    //                 for child in candidate.children.borrow().iter() {
    //                     next_candidates.push(child.clone());
    //                 }
    //             }
    //         }

    //         if found {
    //             candidates = next_candidates;
    //             level -= 1;
    //         } else {
    //             break;
    //         }
    //     }

    //     // Now we can insert at `level`
    //     let new_node = CoverTreeNode::new(p.clone(), level, Weak::new());

    //     // Attach to first valid parent
    //     for parent in candidates {
    //         let dist = parent.city.distance(&p);
    //         if dist <= f32::exp2((level + 1) as f32) {
    //             // new_node.borrow_mut().parent = Rc::downgrade(&parent);
    //             // parent.borrow_mut().children.push(Rc::clone(&new_node));
    //             *new_node.parent.borrow_mut() = Rc::downgrade(&parent);
    //             parent.children.borrow_mut().push(Rc::clone(&new_node));
    //             return true;
    //         }
    //     }
    //     false
    // }

    // /// Recursively find the nearest neighbor
    // fn nearest(self: Rc<Self>, query: &City, best: &mut Option<(City, f32)>) {
    //     let dist = self.city.distance(query);
    //     if best.map(|(_, d)| dist < d).unwrap_or(true) {
    //         *best = Some((self.city.clone(), dist));
    //     }

    //     for child in self.children.borrow().iter() {
    //         let child_dist = child.city.distance(query);
    //         if best.map(|(_, best_d)| (dist - child_dist).abs() <= best_d).unwrap_or(true) {
    //             child.clone().nearest( query, best);
    //         }
    //     }
    // }

    // Try to remove a city from the subtree
    // fn remove(self: Rc<Self>, target: &City) -> bool {
    //     let mut removed = false;

    //     self.children.borrow_mut().retain(|child| {
    //         if child.city == *target {
    //             removed = true;
    //             // Re-parent grandchildren
    //             let mut grandchildren = child.children.borrow_mut();
    //             let grandchildren = std::mem::take(&mut *grandchildren);

    //             for grandchild in grandchildren {
    //                 *grandchild.parent.borrow_mut() = Rc::downgrade(&self);
    //                 self.children.borrow_mut().push(grandchild);
    //             }
    //             false
    //         } else {
    //             true
    //         }
    //     });

    //     if removed {
    //         return true;
    //     }

    //     // Recursively try to remove in children
    //     for child in self.children.borrow().iter() {
    //         if child.clone().remove(target) {
    //             return true;
    //         }
    //     }
    //     false
    // }
}

#[derive(Debug)]
pub struct CoverTree {
    root: Rc<CoverTreeNode>,
}

impl CoverTree {
    pub fn new(root_point: City) -> Self {
        Self {
            root: CoverTreeNode::new(root_point, 0, Weak::new()),
        }
    }

    pub fn insert(&mut self, p: City){
        let root_level = self.root.level.borrow().clone();
        let dist_to_root = self.root.city.distance(&p);
        let new_root_level = f32::log2(dist_to_root).ceil() as i32;
        if new_root_level > root_level{
            for i in (root_level + 1)..=new_root_level {
                let new_root = CoverTreeNode::new(self.root.city.clone(), i, Weak::new());
                *self.root.parent.borrow_mut() = Rc::downgrade(&new_root);
                new_root.children.borrow_mut().push(self.root.clone());
                self.root = new_root;
            }
        }
        assert!(dist_to_root <= f32::exp2(self.root.level.borrow().clone() as f32));
        // it is always possible to find a parent in current_potential_parents that is a valid parent for the new point
        let mut level_to_potential_parents: BTreeMap<i32, Vec<Rc<CoverTreeNode>>> = BTreeMap::new();
        let root_level = self.root.level.borrow().clone();
        level_to_potential_parents.insert(root_level, vec![self.root.clone()]);
        let mut current_level = root_level;
        loop {
            let current_potential_parents = level_to_potential_parents.get(&current_level).unwrap();
            let mut next_potential_parents: Vec<Rc<CoverTreeNode>> = Vec::new();
            for parent in current_potential_parents.iter() {
                for child in parent.children.borrow().iter() {
                    let distance = child.city.distance(&p);
                    // distance <= Σ...
                    if distance < f32::exp2(current_level as f32) {
                        next_potential_parents.push(child.clone());
                    }
                }
            }
            if current_potential_parents.is_empty(){
                break;
            }
            level_to_potential_parents.insert(current_level - 1, next_potential_parents);
            current_level -= 1;
        }
        for (level, potential_parents) in level_to_potential_parents.iter() {
            for parent in potential_parents.iter() {
                let distance = parent.city.distance(&p);
                if distance == 0.0{
                    panic!("Attempting to insert a duplicate point into the cover tree.");
                }
                if distance <= f32::exp2(*level as f32) {
                    parent.clone().insert(p.clone());
                    return;
                }
            }
        }
        panic!("Failed to insert point into cover tree, no valid parent found.");
    }
    


    pub fn nearest_neighbor(&self, query: &City) -> Option<(City, f32)> {
        let mut best_candidate: Option<(City, f32)> = None;

        let mut current_level = self.root.level.borrow().clone();
        let mut current_cover_set = vec![self.root.clone()];
        let mut last_set_min_distance = f32::MAX; // the minimum distance from the last level (current_level + 1) 's cover set to the query point
        loop {
            // filter the current set to exclude points that are too far away from the query point

            // the threashold is based on the maximum possible sum of deviation of the assumed query point from the nodes in the current set
            // since for parent p at level l and any of its child c at level l-1, distance(p, c) <=2^l, 
            // if the query point is a descendant of p, then distance(p, query) <= Σ_i distance(p_i, p_{i-1}) <= Σ_i 2^i for i from -∞ to l < 2^(l+1)
            // therefore, if distance(p, query) >= 2^(l+1), then the query point cannot be a descendant of p, and we can safely exclude p from the current cover set
            let current_cover_set_threshold = f32::exp2((current_level + 1) as f32) + last_set_min_distance;
            current_cover_set.retain(|node| node.city.distance(query) < current_cover_set_threshold);
            if current_cover_set.is_empty() {
                break;
            }

            // update the best candidate based on the current cover set
            for node in current_cover_set.iter() {
                let dist = node.city.distance(query);
                if best_candidate.map(|(_, d)| dist < d).unwrap_or(true) {
                    best_candidate = Some((node.city.clone(), dist));
                }
            }

            // update last_set_min_distance before expanding the children
            let current_set_min_distance = current_cover_set.iter()
                .map(|node| node.city.distance(query))
                .fold(f32::MAX, |a, b| a.min(b));
            assert!(current_set_min_distance <= last_set_min_distance);
            last_set_min_distance = current_set_min_distance;

            // expand the children
            let mut next_cover_set = vec![];
            for node in current_cover_set.iter() {
                for child in node.children.borrow().iter() {                    
                    next_cover_set.push(child.clone());
                }
            }

            // move to the next level
            current_cover_set = next_cover_set;
            current_level -= 1;
        }
        best_candidate
    }

    pub fn remove(&mut self, target: &City) {
        // first find the node to remove
        // assume there is only one node with the target city
        let mut level_to_cover_set: BTreeMap<i32, Vec<Weak<CoverTreeNode>>> = BTreeMap::new();
        let root_level = self.root.level.borrow().clone();
        level_to_cover_set.insert(root_level, vec![Rc::downgrade(&self.root)]);
        let mut current_level = root_level;
        let mut target_node: Option<Rc<CoverTreeNode>> = None;
        loop {
            let current_cover_set = level_to_cover_set.get(&current_level).unwrap();
            if current_cover_set.is_empty() {
                println!("Current cover set is empty at level {}, target city not found in the tree.", current_level);
                break;
            }
            let mut next_cover_set: Vec<Weak<CoverTreeNode>> = Vec::new();            
            for node in current_cover_set.iter() {
                for child in node.upgrade().unwrap().children.borrow().iter() {
                    if target_node.is_none() && child.city == *target {
                        target_node = Some(child.clone());
                    }
                    let distance = child.city.distance(target);
                    if distance < f32::exp2(current_level as f32) {
                        next_cover_set.push(Rc::downgrade(child));
                    }
                }
            }
            if target_node.is_some() {
                break;
            }
            level_to_cover_set.insert(current_level - 1, next_cover_set);
            current_level -= 1;
        }
        let target_level = current_level;
        let target_node = target_node.expect("Target city not found in the cover tree.");
        let parent_node = target_node.parent.borrow().upgrade().expect("Target node has no parent, it must be the root node, which cannot be removed.");
        // take the children out of the target node
        let target_children = std::mem::take(&mut *target_node.children.borrow_mut());
        // unbind the target's children from the target node
        for child in target_children.iter() {
            *child.parent.borrow_mut() = Weak::new();
        }
        // remove the target node from its parent's children list
        // this will also invalidate target in the cover set at level current_level - 1
        parent_node.children.borrow_mut().retain(|child| !Rc::ptr_eq(child, &target_node));
        // the children may need to be promoted different number of levels,
        // and may not be assigned parents at the same level
        let mut remaining_children = target_children;
        // search for the new parent level for each of the target's children
        // until all children have been re-parented to a new parent level, or there are no more levels to search for parents
        for new_parent_level in target_level..{
            // in very rare cases, like the children of the target are at the very edge of the point set, 
            // the new parent level may be greater than the current root level, in which case we need to stack a new root node on top of the current root node
            if new_parent_level > self.root.level.borrow().clone() {
                assert!(new_parent_level == self.root.level.borrow().clone() + 1);
                // stack a new root node on top of the current root node, so that the children may be re-parented to the new root node
                let new_root = CoverTreeNode::new(self.root.city.clone(), new_parent_level, Weak::new());
                *self.root.parent.borrow_mut() = Rc::downgrade(&new_root);
                new_root.children.borrow_mut().push(self.root.clone());
                self.root = new_root;
            }
            // within the distance threshold, the children can be re-parented to the new parent level
            // this is because it satisfies the cover constraint
            let distance_threshold = f32::exp2(new_parent_level as f32);
            let mut new_remaining_children: Vec<Rc<CoverTreeNode>> = Vec::new();
            let potential_parents = level_to_cover_set.get(&new_parent_level).unwrap();
            for child in remaining_children.iter(){
                let mut found_new_parent = false;
                for potential_parent in potential_parents.iter(){
                    // parents may be invalidated because we just removed the target node from the tree
                    let Some(potential_parent) = potential_parent.upgrade() else {
                        continue;
                    };
                    let distance = potential_parent.city.distance(&child.city);
                    if distance <= distance_threshold {
                        *child.parent.borrow_mut() = Rc::downgrade(&potential_parent);
                        potential_parent.children.borrow_mut().push(child.clone());
                        found_new_parent = true;
                        break;
                    }
                }
                if !found_new_parent {
                    // construct a linking node between the current child node and the greater parent level
                    let linking_node = CoverTreeNode::new(child.city.clone(), new_parent_level, Weak::new());
                    *child.parent.borrow_mut() = Rc::downgrade(&linking_node);
                    linking_node.children.borrow_mut().push(child.clone());
                    new_remaining_children.push(linking_node);
                }
            }
            if new_remaining_children.is_empty() {
                break;
            }
            remaining_children = new_remaining_children;
        }
    }
    pub fn print(&self) {
        let mut current_level = self.root.level.borrow().clone();
        let mut current_level_node_sets = vec![vec![self.root.clone()]];
        while current_level_node_sets.len() > 0 {
            println!("Level {}: ", current_level);
            let mut next_level_node_sets = vec![vec![]];
            
            for node_set in current_level_node_sets.iter() {
                let mut next_level_node_set = vec![];
                print!("[");
                for node in node_set.iter() {
                    print!("{:?}", node.city);
                    for child in node.children.borrow().iter() {
                        next_level_node_set.push(child.clone());
                    }
                }
                print!("]");
                println!();
                next_level_node_sets.push(next_level_node_set);
            }
            current_level_node_sets = next_level_node_sets;
            current_level -= 1;
        }
    }
}

