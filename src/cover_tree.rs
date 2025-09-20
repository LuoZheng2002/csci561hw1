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

pub struct CoverTreeNodeChildrenIter<'a, T: Ord + Clone + Distance + std::fmt::Debug> {
    parent_as_child_node: Option<Rc<CoverTreeNode<T>>>,
    non_parent_children_iter: std::slice::Iter<'a, Rc<CoverTreeNode<T>>>,
}

impl<'a, T> Iterator for CoverTreeNodeChildrenIter<'a, T>
where
    T: Ord + Clone + Distance + std::fmt::Debug,
{
    type Item = Rc<CoverTreeNode<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(parent_as_child_node) = self.parent_as_child_node.take() {
            Some(parent_as_child_node)
        } else {
            self.non_parent_children_iter.next().cloned()
        }
    }
}

pub struct CoverTreeNode<T: Ord + Clone + Distance + std::fmt::Debug> {
    point: T,
    level: RefCell<i32>,
    ancestor: RefCell<Weak<CoverTreeNode<T>>>,
    non_self_descendants: RefCell<Vec<Rc<CoverTreeNode<T>>>>,
}

// real node: point, level, neighboring ancestor, neighboring descendants
// dummy node: point, level, corresponding real node, descendants
// maybe no need for dummy node?


impl<T: Ord + Clone + Distance + std::fmt::Debug> CoverTreeNode<T> {
    fn new(point: T, level: i32, parent: Weak<CoverTreeNode<T>>) -> Rc<Self> {
        Rc::new(CoverTreeNode {
            point,
            level: RefCell::new(level),
            ancestor: RefCell::new(parent),
            non_self_descendants: RefCell::new(Vec::new()),
        })
    }
    fn is_leaf(&self) -> bool {
        self.non_self_descendants.borrow().is_empty()
    }
}

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
        // hoist the root to accommodate a faraway new node if necessary
        {
            let dist_to_root = root.point.distance(&p);
            // the potential new root level if the new point is too far away from the root node
            let root_level_lower_bound = f32::log2(dist_to_root).ceil() as i32;
            let mut root_level = root.level.borrow_mut();
            *root_level = (*root_level).max(root_level_lower_bound);
        }
        let root = self.root.clone().unwrap();
        // it is always possible to find a parent in current_potential_parents that is a valid parent for the new point
        let mut level_to_potential_parents: BTreeMap<i32, Vec<Rc<CoverTreeNode<T>>>> = BTreeMap::new();
        let root_level = root.level.borrow().clone();
        level_to_potential_parents.insert(root_level, vec![root.clone()]);
        // populates the potential parents at each level, until there is no more potential parents to explore
        for i in (-root_level..).map(|x| -x){ // an iterator that counts down from root_level
            let current_potential_parents = level_to_potential_parents.get(&i).unwrap();
            let mut next_potential_parents: Vec<Rc<CoverTreeNode<T>>> = Vec::new();
            // filter out the children of current potential parents to be the next potential parents
            for parent in current_potential_parents.iter() {
                // push parent itself as a potential parent at the next level
                // for each child of a parent, if the child's level is i-1, and the distance between the child and the new point p is <= 2^i, then the child is a potential parent at level i-1
                next_potential_parents.push(parent.clone());
                let children = parent.non_self_descendants.borrow();
                for child in children.iter() {
                    if child.level.borrow().clone() != i - 1 {
                        assert!(child.level.borrow().clone() < i - 1);
                        continue;
                    }
                    let distance = child.point.distance(&p);
                    // distance <= Σ...
                    if distance < f32::exp2(i as f32) {
                        next_potential_parents.push(child.clone());
                    }
                }
            }
            if current_potential_parents.is_empty(){
                break;
            }
            level_to_potential_parents.insert(i - 1, next_potential_parents);
        }
        // find the most suitable parent from bottom to top
        for (level, potential_parents) in level_to_potential_parents.iter() {
            for parent in potential_parents.iter() {
                let distance = parent.point.distance(&p);
                if distance == 0.0{
                    panic!("Attempting to insert a duplicate point into the cover tree.");
                }
                // the distance is suitable for the cover constraint
                if distance <= f32::exp2(*level as f32) {
                    // parent.clone().insert_new_point(p.clone());
                    let new_node = CoverTreeNode::new(p.clone(), level - 1, Rc::downgrade(parent));
                    parent.non_self_descendants.borrow_mut().push(new_node);
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
        let root_level = root.level.borrow().clone();
        let mut current_cover_set = vec![root.clone()];
        let mut last_set_min_distance = f32::MAX; // the minimum distance from the last level (current_level + 1) 's cover set to the query point
        for i in (-root_level..).map(|x| -x) {
            // filter the current set to exclude points that are too far away from the query point

            // the threashold is based on the maximum possible sum of deviation of the assumed query point from the nodes in the current set
            // since for parent p at level l and any of its child c at level l-1, distance(p, c) <=2^l, 
            // if the query point is a descendant of p, then distance(p, query) <= Σ_i distance(p_i, p_{i-1}) <= Σ_i 2^i for i from -∞ to l < 2^(l+1)
            // therefore, if distance(p, query) >= 2^(l+1), then the query point cannot be a descendant of p, and we can safely exclude p from the current cover set
            let current_cover_set_threshold = f32::exp2((i + 1) as f32) + last_set_min_distance;
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
                // push the node itself as a potential candidate at the next level
                next_cover_set.push(node.clone());
                // push all children of the node as potential candidates at the next level
                let children = node.non_self_descendants.borrow();
                for child in children.iter() {
                    if child.level.borrow().clone() != i - 1 {
                        assert!(child.level.borrow().clone() < i - 1);
                        continue;
                    }
                    next_cover_set.push(child.clone());
                }
            }
            // move to the next level
            current_cover_set = next_cover_set;
        }
        Some(best_candidate.unwrap())
    }

    pub fn remove(&mut self, target: &T) {
        let Some(root) = self.root.as_ref() else {
            println!("Cover tree is empty, cannot remove target point.");
            panic!("Cover tree is empty, cannot remove target point.");
        };
        // first find the node to remove
        // assume there is only one node with the target point
        let mut level_to_cover_set: BTreeMap<i32, Vec<Weak<CoverTreeNode<T>>>> = BTreeMap::new();
        let root_level = root.level.borrow().clone();
        level_to_cover_set.insert(root_level, vec![Rc::downgrade(&root)]);
        let target_node: Rc<CoverTreeNode<T>> = 'found_target_node_label: {
            for i in (-root_level..).map(|x| -x) {
                let current_cover_set = level_to_cover_set.get(&i).expect("Cannot find the query, reached the bottom of the tree.");
                assert!(!current_cover_set.is_empty());
                let mut next_cover_set: Vec<Weak<CoverTreeNode<T>>> = Vec::new();
                // search for the target node in the children of the current cover set
                for node in current_cover_set.iter() {
                    let node = node.upgrade().unwrap();
                    // push the node itself as a potential candidate at the next level
                    next_cover_set.push(Rc::downgrade(&node));
                    // push all children of the node as potential candidates at the next level
                    let children = node.non_self_descendants.borrow();
                    for child in children.iter() {
                        if child.point == *target {
                            break 'found_target_node_label child.clone();
                        }
                        if child.level.borrow().clone() != i - 1 {
                            assert!(child.level.borrow().clone() < i - 1);
                            continue;
                        }
                        let distance = child.point.distance(target);
                        if distance < f32::exp2(i as f32) {
                            next_cover_set.push(Rc::downgrade(child));
                        }
                    }
                }
                level_to_cover_set.insert(i - 1, next_cover_set);
            }
            panic!("Target point not found in the cover tree.");
        };
        // take the children out of the target node
        let target_children = std::mem::take(&mut *target_node.non_self_descendants.borrow_mut());
        // unbind the target's children from the target node
        for child in target_children.iter() {
            *child.ancestor.borrow_mut() = Weak::new();
        }
        if let Some(parent_node) = target_node.ancestor.borrow().upgrade(){
            // remove the target node from its parent's children list
            // this will also invalidate target in the cover set at level current_level - 1
            assert_ne!(&parent_node.point, &target_node.point);
            let mut parent_children = parent_node.non_self_descendants.borrow_mut();
            let num_descendants_before_removal = parent_children.len();
            parent_children.retain(|child| !Rc::ptr_eq(child, &target_node));
            assert_eq!(num_descendants_before_removal - 1, parent_children.len());
        } else {
            // the target node has no parent, it must be the root node
            assert!(Rc::ptr_eq(&root, &target_node));
        };       
        
        // the children may need to be promoted different number of levels,
        // and may not be assigned parents at the same level
        let mut remaining_children: Vec<_> = target_children.iter().cloned().collect();
        // search for the new parent level for each of the target's children
        // until all children have been re-parented to a new parent level, or there are no more levels to search for parents
        for new_parent_level in target_level.. {
            let root = self.root.clone().unwrap();
            // in very rare cases, like the children of the target are at the very edge of the point set, 
            // the new parent level may be greater than the current root level, in which case we need to stack a new root node on top of the current root node
            if new_parent_level > root.level.borrow().clone() {
                assert!(new_parent_level == root.level.borrow().clone() + 1);
                // stack a new root node on top of the current root node, so that the children may be re-parented to the new root node
                let new_root = CoverTreeNode::new(root.point.clone(), new_parent_level, Weak::new());
                *root.ancestor.borrow_mut() = Rc::downgrade(&new_root);
                // new root's children is itself with one fewer level
                new_root.non_self_descendants.borrow_mut().insert_self(root.clone());
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
                        // place the child under the potential parent
                        *child.ancestor.borrow_mut() = Rc::downgrade(&potential_parent);
                        // assert level
                        assert!(potential_parent.level.borrow().clone() == child.level.borrow().clone() + 1);
                        potential_parent.non_self_descendants.borrow_mut().insert_new_node(&potential_parent,child.clone());
                        found_new_parent = true;
                        break;
                    }
                }
                if !found_new_parent {
                    // construct a linking node between the current child node and the greater parent level
                    let linking_node = CoverTreeNode::new(child.point.clone(), new_parent_level, Weak::new());
                    *child.ancestor.borrow_mut() = Rc::downgrade(&linking_node);
                    linking_node.non_self_descendants.borrow_mut().insert_self(child.clone());
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
            let mut next_level_node_sets: Vec<Vec<Rc<CoverTreeNode<T>>>> = vec![];
            
            for node_set in current_level_node_sets.iter() {
                // print current node set
                print!("[");
                for node in node_set.iter() {
                    // print current node point
                    print!("{:?}, ", node.point);
                    // prepare next level node set
                    let mut next_level_node_set = vec![];                    
                    for child in node.non_self_descendants.borrow().iter() {
                        next_level_node_set.push(child.clone());
                    }
                    if !next_level_node_set.is_empty() {
                        next_level_node_sets.push(next_level_node_set);
                    }
                }
                print!("]");
                println!();                
            }
            current_level_node_sets = next_level_node_sets;
            current_level -= 1;
        }
    }
    pub fn assert_valid_cover_tree(&self) -> Result<(), String>{
        let Some(root) = self.root.as_ref() else {
            return Ok(());
        };
        let mut current_level = root.level.borrow().clone();
        let mut current_level_nodes = Vec::new();
        current_level_nodes.push(root.clone());
        let mut current_level_value_set: BTreeSet<_> = current_level_nodes.iter().map(|node| node.point.clone()).collect();
        while !current_level_nodes.is_empty() {
            let mut next_level_nodes = Vec::new();
            for node in current_level_nodes.iter() {
                for child in node.non_self_descendants.borrow().iter() {
                    let distance = node.point.distance(&child.point);
                    // test the covering property
                    if distance > f32::exp2(current_level as f32) {
                        return Err(format!("Cover tree cover constraint violated: distance between parent {:?} and child {:?} is {}, which exceeds the threshold of {}", node.point, child.point, distance, f32::exp2(current_level as f32)));
                    }
                    next_level_nodes.push(child.clone());
                }
            }
            // test the separation property
            for (i, node_a) in current_level_nodes.iter().enumerate() {
                for node_b in current_level_nodes.iter().skip(i + 1) {
                    let distance = node_a.point.distance(&node_b.point);
                    if distance <= f32::exp2(current_level as f32) {
                        return Err(format!("Cover tree separation constraint violated: distance between nodes {:?} and {:?} is {}, which does not exceed the threshold of {}", node_a.point, node_b.point, distance, f32::exp2(current_level as f32)));
                    }
                }
            }
            let next_level_value_set = next_level_nodes.iter().map(|node| node.point.clone()).collect::<BTreeSet<_>>();
            // test the nesting property
            // assert!(next_level_value_set.is_superset(&current_level_value_set));
            if !next_level_value_set.is_empty() && !next_level_value_set.is_superset(&current_level_value_set){
                return Err(format!("Nesting property violated at level {}. Next level value set: {:?}, Current level value set: {:?}", current_level, next_level_value_set, current_level_value_set));
            }           
            // move to the next level
            current_level_nodes = next_level_nodes;
            current_level_value_set = next_level_value_set;
            current_level -= 1;
        }
        Ok(())
    }
}
