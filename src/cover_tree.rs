use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::vec;

pub trait Distance {
    fn distance(&self, other: &Self) -> f32;
}
impl Distance for u32 {
    fn distance(&self, other: &Self) -> f32 {
        (*self as i32 - *other as i32).abs() as f32
    }
}

impl Distance for i32 {
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

impl<T: Ord + Clone + Distance + std::fmt::Debug> CoverTreeNode<T> {
    fn new(point: T, level: i32, parent: Weak<CoverTreeNode<T>>) -> Rc<Self> {
        Rc::new(CoverTreeNode {
            point,
            level: RefCell::new(level),
            ancestor: RefCell::new(parent),
            non_self_descendants: RefCell::new(Vec::new()),
        })
    }
}

pub struct CoverTree<T: Ord + Clone + Distance + std::fmt::Debug> {
    root: Option<Rc<CoverTreeNode<T>>>,
}

impl<T: Ord + Clone + Distance + std::fmt::Debug> CoverTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, p: T) {
        // if there is no root node, create a new root node with the point p and level 0
        let Some(root) = self.root.as_ref() else {
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
        let mut level_to_potential_parents: BTreeMap<i32, Vec<Rc<CoverTreeNode<T>>>> =
            BTreeMap::new();
        let root_level = root.level.borrow().clone();
        level_to_potential_parents.insert(root_level, vec![root.clone()]);
        // populates the potential parents at each level, until there is no more potential parents to explore
        for i in (-root_level..).map(|x| -x) {
            // an iterator that counts down from root_level
            if i == -1000 {
                panic!("Infinite loop detected when inserting point into cover tree.");
            }
            let current_potential_parents = level_to_potential_parents.get(&i).unwrap();
            let mut next_potential_parents: Vec<Rc<CoverTreeNode<T>>> = Vec::new();
            // filter out the children of current potential parents to be the next potential parents
            for parent in current_potential_parents.iter() {
                let parent_p_distance = parent.point.distance(&p);
                if parent_p_distance < f32::exp2(i as f32) {
                    // the parent itself can be a valid parent for the new point
                    next_potential_parents.push(parent.clone());
                }
                let children = parent.non_self_descendants.borrow();
                for child in children.iter() {
                    let child_level = child.level.borrow().clone();
                    if child_level == i - 1 {
                        // needs to be pushed to the next potential parents if it satisfies the cover constraint
                        let distance = child.point.distance(&p);
                        // distance <= Σ...
                        if distance < f32::exp2(i as f32) {
                            next_potential_parents.push(child.clone());
                        }
                    }
                }
            }
            if next_potential_parents.is_empty() {
                break;
            }

            level_to_potential_parents.insert(i - 1, next_potential_parents);
        }
        // find the most suitable parent from bottom to top
        for (level, potential_parents) in level_to_potential_parents.iter() {
            for parent in potential_parents.iter() {
                let distance = parent.point.distance(&p);
                if distance == 0.0 {
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
        let mut best_candidate = root.clone();
        let root_level = root.level.borrow().clone();
        let mut current_cover_set = vec![root.clone()];
        let mut best_distance = best_candidate.point.distance(query); // the minimum distance from the last level (current_level + 1) 's cover set to the query point

        for i in (-root_level..).map(|x| -x) {
            if i == -1000 {
                panic!("Infinite loop detected when searching for nearest neighbor in cover tree.");
            }
            let mut has_remaining_children = false;

            let mut next_cover_set: Vec<Rc<CoverTreeNode<T>>> = Vec::new();
            // let mut min_distance_in_next_set = f32::MAX;
            for parent in current_cover_set.iter() {
                next_cover_set.push(parent.clone());
                for child in parent.non_self_descendants.borrow().iter() {
                    let child_level = child.level.borrow().clone();
                    if child_level == i - 1 {
                        let distance = child.point.distance(query);
                        if distance < best_distance {
                            best_distance = distance;
                            best_candidate = child.clone();
                        }
                        next_cover_set.push(child.clone());
                    }
                    if child_level <= i - 1 {
                        has_remaining_children = true;
                    }
                }
            }
            let threshold = f32::exp2(i as f32) + best_distance;
            next_cover_set.retain(|node| node.point.distance(query) < threshold);
            if next_cover_set.is_empty() {
                break;
            }
            if !has_remaining_children {
                break;
            }
            current_cover_set = next_cover_set;
        }
        // Some(best_candidate.expect("Reached the bottom of the cover tree but no candidate found, which should not happen."))
        Some((best_candidate.point.clone(), best_distance))
    }

    pub fn remove(&mut self, target: &T) {
        let Some(root) = self.root.clone() else {
            panic!("Cover tree is empty, cannot remove target point.");
        };
        // first find the node to remove
        // assume there is only one node with the target point
        let mut level_to_cover_set: BTreeMap<i32, Vec<Weak<CoverTreeNode<T>>>> = BTreeMap::new();
        let root_level = root.level.borrow().clone();
        level_to_cover_set.insert(root_level, vec![Rc::downgrade(&root)]);
        let mut target_node_and_lowest_child: Option<(Rc<CoverTreeNode<T>>, i32)> = None;
        for i in (-root_level..).map(|x| -x) {
            if i == -10 {
                panic!("Infinite loop detected when searching for target point in cover tree.");
            }

            let current_cover_set = level_to_cover_set
                .get(&i)
                .expect("Should be filled in the last iteration.");
            if current_cover_set.is_empty() {
                println!("Reached the bottom of the cover tree but cannot find the target point.");
                break;
            }
            if target_node_and_lowest_child.is_none() {
                target_node_and_lowest_child = current_cover_set.iter().find_map(|weak_node| {
                    let node = weak_node.upgrade()?;
                    if node.point == *target {
                        let lowest_child = {
                            let children = node.non_self_descendants.borrow();
                            children
                                .iter()
                                .map(|child| {
                                    assert!(
                                        child.level.borrow().clone() < node.level.borrow().clone()
                                    );
                                    child.level.borrow().clone()
                                })
                                .min()
                                .unwrap_or_else(|| {
                                    println!("Target node has no children.");
                                    node.level.borrow().clone()
                                })
                        };
                        Some((node, lowest_child))
                    } else {
                        None
                    }
                });
            }
            if let Some((node, lowest_child_level)) = target_node_and_lowest_child.as_ref() {
                if i - 1 == *lowest_child_level {
                    // we have reached the lowest child level of the target node, no need to go further down
                    break;
                } else if i - 1 < *lowest_child_level {
                    assert!(
                        node.non_self_descendants.borrow().is_empty(),
                        "i - 1: {}, lowest_child_level: {}",
                        i - 1,
                        lowest_child_level
                    );
                    break;
                }
            }
            let mut has_remaining_children = true;
            assert!(!current_cover_set.is_empty());
            let mut next_cover_set: Vec<Weak<CoverTreeNode<T>>> = Vec::new();
            // search for the target node in the children of the current cover set
            // even if we found the target node, we still need to populate the next cover set for re-parenting the target's children
            for node in current_cover_set.iter() {
                let node = node.upgrade().unwrap();
                // push the node itself as a potential candidate at the next level
                let parent_target_distance = node.point.distance(target);
                if parent_target_distance <= f32::exp2(i as f32) {
                    next_cover_set.push(Rc::downgrade(&node));
                }
                // push all children of the node as potential candidates at the next level
                let children = node.non_self_descendants.borrow();
                for child in children.iter() {
                    let child_level = child.level.borrow().clone();
                    if child_level == i - 1 {
                        let distance = child.point.distance(target);
                        if distance <= f32::exp2(i as f32) {
                            next_cover_set.push(Rc::downgrade(child));
                        }
                    }
                    if child_level <= i - 1 {
                        has_remaining_children = true;
                    }
                }
            }
            level_to_cover_set.insert(i - 1, next_cover_set);
            if !has_remaining_children {
                println!("Reached the bottom of the cover tree but cannot find the target point.");
                // serves as an early stop in case the value to be removed is extremely close to one of the nodes, but actually not in the tree.
                break;
            }
        }
        drop(root);
        let (target_node, lowest_child_level) =
            target_node_and_lowest_child.expect("Cannot find the target point in the cover tree.");
        // take the children out of the target node
        let mut target_children =
            std::mem::take(&mut *target_node.non_self_descendants.borrow_mut());
        // unbind the target's children from the target node
        for child in target_children.iter() {
            *child.ancestor.borrow_mut() = Weak::new();
        }
        // process the target's parent node if there is one
        if let Some(parent_node) = target_node.ancestor.borrow().upgrade() {
            // remove the target node from its parent's children list
            // this will also invalidate target in the cover set at level current_level - 1
            assert_ne!(&parent_node.point, &target_node.point);
            let mut parent_children = parent_node.non_self_descendants.borrow_mut();
            let num_descendants_before_removal = parent_children.len();
            parent_children.retain(|child| !Rc::ptr_eq(child, &target_node));
            assert_eq!(num_descendants_before_removal - 1, parent_children.len());
        } else {
            // the target node has no parent, it must be the root node
            assert!(Rc::ptr_eq(&self.root.clone().unwrap(), &target_node));
            // drop the root node so that its reference count drops to 0 and it can be deallocated
            self.root = None;
            let highest_target_child = target_children
                .iter()
                .max_by_key(|child| child.level.borrow().clone())
                .cloned();
            let Some(highest_target_child) = highest_target_child else {
                // the target node has no children, the tree is now empty
                self.root = None;
                return;
            };
            let num_children_before = target_children.len();
            target_children.retain(|child| !Rc::ptr_eq(child, &highest_target_child));
            assert_eq!(num_children_before - 1, target_children.len());
            // promote one of the target's children to be the new root node
            assert!(highest_target_child.ancestor.borrow().upgrade().is_none()); // its parent was removed in the last step
            self.root = Some(highest_target_child.clone());
        };
        // let target_level = target_node.level.borrow().clone();
        if !target_node.non_self_descendants.borrow().is_empty() {
            assert_eq!(
                *level_to_cover_set.first_key_value().unwrap().0,
                lowest_child_level + 1
            );
        }
        drop(target_node);

        // let mut remaining_children = target_children;

        // search for the new parent level for each of the target's children
        // until all children have been re-parented to a new parent level, or there are no more levels to search for parents
        for child in target_children.iter() {
            let mut valid_parent_and_parent_level: Option<(Rc<CoverTreeNode<T>>, i32)> = None;
            let child_level = child.level.borrow().clone();
            for new_parent_level in child_level + 1.. {
                if let Some(potential_parents) = level_to_cover_set.get(&new_parent_level) {
                    // within the distance threshold, the children can be re-parented to the new parent level
                    // this is because it satisfies the cover constraint
                    let distance_threshold = f32::exp2(new_parent_level as f32);
                    for potential_parent in potential_parents.iter() {
                        // parents may be invalidated because we just removed the target node from the tree
                        let Some(potential_parent) = potential_parent.upgrade() else {
                            continue;
                        };
                        let distance = potential_parent.point.distance(&child.point);
                        if distance <= distance_threshold {
                            valid_parent_and_parent_level =
                                Some((potential_parent, new_parent_level));
                            break;
                        }
                    }
                }
                // if we did not find a valid parent, use root as fallback
                valid_parent_and_parent_level = valid_parent_and_parent_level.or_else(|| {
                    let root = self
                        .root
                        .clone()
                        .expect("Cover tree must have a root node at this point.");
                    {
                        let mut root_level = root.level.borrow_mut();
                        if new_parent_level > *root_level {
                            *root_level = (*root_level).max(new_parent_level);
                        }
                    }
                    let distance = root.point.distance(&child.point);
                    if distance <= f32::exp2(new_parent_level as f32) {
                        Some((root, new_parent_level))
                    } else {
                        None
                    }
                });
                if valid_parent_and_parent_level.is_some() {
                    break;
                }
            }
            let (valid_parent, valid_parent_level) = valid_parent_and_parent_level
                .expect("Failed to find a valid parent for a child of the removed node.");
            // place the child under the potential parent
            *child.ancestor.borrow_mut() = Rc::downgrade(&valid_parent);
            let old_child_level = child.level.borrow().clone();
            let new_child_level = valid_parent_level - 1;
            for child_covered_level in old_child_level + 1..=new_child_level {
                let cover_set = level_to_cover_set.entry(child_covered_level).or_default();
                cover_set.push(Rc::downgrade(child));
            }
            *child.level.borrow_mut() = new_child_level;
            // assert level
            assert!(
                valid_parent.level.borrow().clone() > child.level.borrow().clone(),
                "valid parent level: {}, child level: {}",
                valid_parent.level.borrow().clone(),
                child.level.borrow().clone()
            );
            valid_parent
                .non_self_descendants
                .borrow_mut()
                .push(child.clone());
        }
    }
    pub fn print(&self) {
        let Some(root) = self.root.as_ref() else {
            println!("Cover tree is empty.");
            return;
        };
        println!("Cover tree: -------------------------------");
        let mut current_level = root.level.borrow().clone();
        let mut current_level_node_sets = vec![vec![root.clone()]];
        let mut max_num_children = u32::MIN;
        loop {
            let mut has_remaining_children = false;
            let max_set_size = current_level_node_sets
                .iter()
                .map(|set| set.len())
                .max()
                .unwrap_or(0);
            let num_nodes = current_level_node_sets
                .iter()
                .map(|set| set.len())
                .sum::<usize>();
            println!(
                "Level {}: max children count: {}, num nodes: {}",
                current_level, max_set_size, num_nodes
            );
            max_num_children = max_num_children.max(max_set_size as u32);
            let mut next_level_node_sets: Vec<Vec<Rc<CoverTreeNode<T>>>> = vec![];

            for node_set in current_level_node_sets.iter() {
                // print current node set
                print!("[");
                for node in node_set.iter() {
                    // print current node point
                    print!("{:?}, ", node.point);
                    // prepare next level node set
                    let mut next_level_node_set = vec![];
                    // add the node itself to the next level node set
                    next_level_node_set.push(node.clone());
                    for child in node.non_self_descendants.borrow().iter() {
                        let child_level = child.level.borrow().clone();
                        if child_level == current_level - 1 {
                            next_level_node_set.push(child.clone());
                        }
                        if child_level <= current_level - 1 {
                            has_remaining_children = true;
                        }
                    }
                    if !next_level_node_set.is_empty() {
                        next_level_node_sets.push(next_level_node_set);
                    }
                }
                print!("]");
            }
            println!();
            if !has_remaining_children {
                break;
            }
            current_level_node_sets = next_level_node_sets;
            current_level -= 1;
        }
        println!("max number of children at any level: {}", max_num_children);
        println!("------------------------------------------");
    }
    pub fn assert_valid_cover_tree(&self) -> Result<(), String> {
        let Some(root) = self.root.as_ref() else {
            return Ok(());
        };
        let root_level = root.level.borrow().clone();
        let mut current_level_nodes = vec![root.clone()];
        for current_level in (-root_level..).map(|x| -x) {
            let mut has_remaining_children = false;
            if current_level == -1000 {
                panic!("Infinite loop detected when validating cover tree.");
            }
            if current_level_nodes.is_empty() {
                break;
            }
            let mut next_level_nodes = Vec::new();
            for node in current_level_nodes.iter() {
                // add the node itself to the next level nodes
                next_level_nodes.push(node.clone());
                let children = node.non_self_descendants.borrow();
                for child in children.iter() {
                    let child_level = child.level.borrow().clone();
                    if child_level == current_level - 1 {
                        next_level_nodes.push(child.clone());

                        // test the covering property
                        let distance = node.point.distance(&child.point);
                        if distance > f32::exp2(current_level as f32) {
                            return Err(format!(
                                "Cover tree cover constraint violated: distance between parent {:?} and child {:?} is {}, which exceeds the threshold of {}",
                                node.point,
                                child.point,
                                distance,
                                f32::exp2(current_level as f32)
                            ));
                        }
                    }
                    if child_level <= current_level - 1 {
                        has_remaining_children = true;
                    }
                }
            }
            for (i, node_a) in current_level_nodes.iter().enumerate() {
                for node_b in current_level_nodes.iter().skip(i + 1) {
                    let distance = node_a.point.distance(&node_b.point);
                    if distance <= f32::exp2(current_level as f32) {
                        return Err(format!(
                            "Cover tree separation constraint violated: distance between nodes {:?} and {:?} is {}, which does not exceed the threshold of {}",
                            node_a.point,
                            node_b.point,
                            distance,
                            f32::exp2(current_level as f32)
                        ));
                    }
                }
            }
            if !has_remaining_children {
                break;
            }
            // move to the next level
            current_level_nodes = next_level_nodes;
        }
        Ok(())
    }
}
