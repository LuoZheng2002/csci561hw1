use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::{Rc, Weak};

pub trait Distance{
    fn distance(&self, other: &Self) -> f32;
}



impl Distance for i32{    
    fn distance(&self, other: &Self) -> f32 {
        (*self - *other).abs() as f32
    }
}


#[derive(Debug)]
pub struct CoverTreeNode<T: Ord + Clone + Distance + std::fmt::Debug> {
    point: T,
    level: RefCell<i32>,
    parent: RefCell<Weak<CoverTreeNode<T>>>,
    children: RefCell<Vec<Rc<CoverTreeNode<T>>>>,
}

impl<T: Ord + Clone + Distance + std::fmt::Debug> CoverTreeNode<T> {
    fn new(point: T, level: i32, parent: Weak<CoverTreeNode<T>>) -> Rc<Self> {
        Rc::new(CoverTreeNode {
            point,
            level: RefCell::new(level),
            parent: RefCell::new(parent),
            children: RefCell::new(vec![]),
        })
    }

    /// Insert a new point into this node or its descendants
    /// Returns true if there is a parent found
    /// 
    fn insert(self: Rc<Self>, p: T) {
        self.children
            .borrow_mut()
            .push(CoverTreeNode::new(p, self.level.borrow().clone() - 1, Rc::downgrade(&self)));
    }
}

#[derive(Debug)]
pub struct CoverTree<T: Ord + Clone + Distance + std::fmt::Debug> {
    root: Option<Rc<CoverTreeNode<T>>>,
}

impl<T: Ord + Clone + Distance + std::fmt::Debug> CoverTree<T> {
    pub fn new() -> Self {
        Self {
            root: None,
        }
    }

    pub fn insert(&mut self, p: T){
        // if there is no root node, create a new root node with the point p and level 0
        let Some(root) = self.root.as_ref() else{
            self.root = Some(CoverTreeNode::new(p, 0, Weak::new()));
            return;
        };
        let root_level = root.level.borrow().clone();
        let dist_to_root = root.point.distance(&p);
        // the potential new root level if the new point is too far away from the root node
        let new_root_level = f32::log2(dist_to_root).ceil() as i32;
        if new_root_level > root_level{
            let mut current_root = root.clone();
            let root_value = current_root.point.clone();
            for i in (root_level + 1)..=new_root_level {
                let new_root = CoverTreeNode::new(root_value.clone(), i, Weak::new());
                *current_root.parent.borrow_mut() = Rc::downgrade(&new_root);
                new_root.children.borrow_mut().push(current_root.clone());
                current_root = new_root;
            }
            self.root = Some(current_root);
        }
        let root = self.root.clone().unwrap();
        assert!(dist_to_root <= f32::exp2(root.level.borrow().clone() as f32));
        // it is always possible to find a parent in current_potential_parents that is a valid parent for the new point
        let mut level_to_potential_parents: BTreeMap<i32, Vec<Rc<CoverTreeNode<T>>>> = BTreeMap::new();
        let root_level = root.level.borrow().clone();
        level_to_potential_parents.insert(root_level, vec![root.clone()]);
        let mut current_level = root_level;
        loop {
            let current_potential_parents = level_to_potential_parents.get(&current_level).unwrap();
            let mut next_potential_parents: Vec<Rc<CoverTreeNode<T>>> = Vec::new();
            for parent in current_potential_parents.iter() {
                for child in parent.children.borrow().iter() {
                    let distance = child.point.distance(&p);
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
                let distance = parent.point.distance(&p);
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
    


    pub fn nearest_neighbor(&self, query: &T) -> Option<(T, f32)> {
        // retrieve the root node, if there is no root node, return None
        let Some(root) = self.root.as_ref() else {
            return None;
        };
        let mut best_candidate: Option<(T, f32)> = None;

        let mut current_level = root.level.borrow().clone();
        let mut current_cover_set = vec![root.clone()];
        let mut last_set_min_distance = f32::MAX; // the minimum distance from the last level (current_level + 1) 's cover set to the query point
        loop {
            // filter the current set to exclude points that are too far away from the query point

            // the threashold is based on the maximum possible sum of deviation of the assumed query point from the nodes in the current set
            // since for parent p at level l and any of its child c at level l-1, distance(p, c) <=2^l, 
            // if the query point is a descendant of p, then distance(p, query) <= Σ_i distance(p_i, p_{i-1}) <= Σ_i 2^i for i from -∞ to l < 2^(l+1)
            // therefore, if distance(p, query) >= 2^(l+1), then the query point cannot be a descendant of p, and we can safely exclude p from the current cover set
            let current_cover_set_threshold = f32::exp2((current_level + 1) as f32) + last_set_min_distance;
            current_cover_set.retain(|node| node.point.distance(query) < current_cover_set_threshold);
            if current_cover_set.is_empty() {
                break;
            }

            // update the best candidate based on the current cover set
            for node in current_cover_set.iter() {
                let dist = node.point.distance(query);
                if best_candidate.as_ref().map(|(_, d)| dist < *d).unwrap_or(true) {
                    best_candidate = Some((node.point.clone(), dist));
                }
            }

            // update last_set_min_distance before expanding the children
            let current_set_min_distance = current_cover_set.iter()
                .map(|node| node.point.distance(query))
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

    pub fn remove(&mut self, target: &T) {
        let Some(root) = self.root.as_ref() else {
            println!("Cover tree is empty, cannot remove target point.");
            return;
        };
        // first find the node to remove
        // assume there is only one node with the target point
        let mut level_to_cover_set: BTreeMap<i32, Vec<Weak<CoverTreeNode<T>>>> = BTreeMap::new();
        let root_level = root.level.borrow().clone();
        level_to_cover_set.insert(root_level, vec![Rc::downgrade(&root)]);
        let mut current_level = root_level;
        let mut target_node: Option<Rc<CoverTreeNode<T>>> = None;
        loop {
            let current_cover_set = level_to_cover_set.get(&current_level).unwrap();
            if current_cover_set.is_empty() {
                println!("Current cover set is empty at level {}, target point not found in the tree.", current_level);
                break;
            }
            let mut next_cover_set: Vec<Weak<CoverTreeNode<T>>> = Vec::new();            
            for node in current_cover_set.iter() {
                for child in node.upgrade().unwrap().children.borrow().iter() {
                    if target_node.is_none() && child.point == *target {
                        target_node = Some(child.clone());
                    }
                    let distance = child.point.distance(target);
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
        let target_node = target_node.expect("Target point not found in the cover tree.");
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
            let root = self.root.clone().unwrap();
            // in very rare cases, like the children of the target are at the very edge of the point set, 
            // the new parent level may be greater than the current root level, in which case we need to stack a new root node on top of the current root node
            if new_parent_level > root.level.borrow().clone() {
                assert!(new_parent_level == root.level.borrow().clone() + 1);
                // stack a new root node on top of the current root node, so that the children may be re-parented to the new root node
                let new_root = CoverTreeNode::new(root.point.clone(), new_parent_level, Weak::new());
                *root.parent.borrow_mut() = Rc::downgrade(&new_root);
                new_root.children.borrow_mut().push(root.clone());
                self.root = Some(new_root);
            }
            // within the distance threshold, the children can be re-parented to the new parent level
            // this is because it satisfies the cover constraint
            let distance_threshold = f32::exp2(new_parent_level as f32);
            let mut new_remaining_children: Vec<Rc<CoverTreeNode<T>>> = Vec::new();
            let potential_parents = level_to_cover_set.get(&new_parent_level).unwrap();
            for child in remaining_children.iter(){
                let mut found_new_parent = false;
                for potential_parent in potential_parents.iter(){
                    // parents may be invalidated because we just removed the target node from the tree
                    let Some(potential_parent) = potential_parent.upgrade() else {
                        continue;
                    };
                    let distance = potential_parent.point.distance(&child.point);
                    if distance <= distance_threshold {
                        *child.parent.borrow_mut() = Rc::downgrade(&potential_parent);
                        potential_parent.children.borrow_mut().push(child.clone());
                        found_new_parent = true;
                        break;
                    }
                }
                if !found_new_parent {
                    // construct a linking node between the current child node and the greater parent level
                    let linking_node = CoverTreeNode::new(child.point.clone(), new_parent_level, Weak::new());
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
        let Some(root) = self.root.as_ref() else {
            println!("Cover tree is empty.");
            return;
        };
        let mut current_level = root.level.borrow().clone();
        let mut current_level_node_sets = vec![vec![root.clone()]];
        while current_level_node_sets.len() > 0 {
            println!("Level {}: ", current_level);
            let mut next_level_node_sets = vec![vec![]];
            
            for node_set in current_level_node_sets.iter() {
                let mut next_level_node_set = vec![];
                print!("[");
                for node in node_set.iter() {
                    print!("{:?}", node.point);
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
    pub fn assert_valid_cover_tree(&self){
        let Some(root) = self.root.as_ref() else {
            return;
        };
        let mut current_level = root.level.borrow().clone();
        let mut current_level_nodes = Vec::new();
        current_level_nodes.push(root.clone());
        let mut current_level_value_set: BTreeSet<_> = current_level_nodes.iter().map(|node| node.point.clone()).collect();
        while !current_level_nodes.is_empty() {
            let mut next_level_nodes = Vec::new();
            for node in current_level_nodes.iter() {
                for child in node.children.borrow().iter() {
                    let distance = node.point.distance(&child.point);
                    // test the covering property
                    assert!(distance <= f32::exp2(current_level as f32), 
                        "Cover tree cover constraint violated: distance between parent {:?} and child {:?} is {}, which exceeds the threshold of {}", 
                        node.point, child.point, distance, f32::exp2(current_level as f32));
                    next_level_nodes.push(child.clone());
                }
            }
            // test the separation property
            for (i, node_a) in current_level_nodes.iter().enumerate() {
                for node_b in current_level_nodes.iter().skip(i + 1) {
                    let distance = node_a.point.distance(&node_b.point);
                    assert!(distance > f32::exp2(current_level as f32), 
                        "Cover tree separation constraint violated: distance between nodes {:?} and {:?} is {}, which does not exceed the threshold of {}", 
                        node_a.point, node_b.point, distance, f32::exp2(current_level as f32));
                }
            }
            let next_level_value_set = next_level_nodes.iter().map(|node| node.point.clone()).collect::<BTreeSet<_>>();
            // test the nesting property
            assert!(next_level_value_set.is_superset(&current_level_value_set));            
           
            // move to the next level
            current_level_nodes = next_level_nodes;
            current_level_value_set = next_level_value_set;
            current_level -= 1;
        }
    }
}
