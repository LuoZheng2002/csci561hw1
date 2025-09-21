use core::panic;

use rust::cover_tree::{CoverTree, Distance};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Point2D {
    x: i32,
    y: i32,
}

impl Distance for Point2D {
    fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }
}

impl std::fmt::Display for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl std::fmt::Debug for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[test]
fn test_insert_and_nearest_neighbor() {
    let mut tree = CoverTree::<Point2D>::new();
    let points = vec![
        Point2D { x: 0, y: 0 },
        Point2D { x: 10, y: 10 },
        Point2D { x: -10, y: -10 },
        Point2D { x: 5, y: 5 },
        Point2D { x: 1, y: 1 },
    ];
    for p in &points {
        tree.print();
        println!("Inserting point: {}", p);
        tree.insert(p.clone());
        if let Err(e) = tree.assert_valid_cover_tree() {
            println!(
                "Cover tree validation failed after inserting point {}: {}",
                p, e
            );
            tree.print();
            panic!();
        }
    }

    let query = Point2D { x: 0, y: 1 };
    let (nn, dist) = tree.nearest_neighbor(&query).unwrap();
    assert_eq!(nn, Point2D { x: 0, y: 0 });
    assert!((dist - 1.0).abs() < 1e-5);
}

#[test]
#[should_panic(expected = "duplicate")]
fn test_insert_duplicate_should_panic() {
    let mut tree = CoverTree::<Point2D>::new();
    let p = Point2D { x: 1, y: 2 };
    tree.insert(p.clone());
    tree.insert(p); // should panic
}

#[test]
fn test_remove_node() {
    let mut tree = CoverTree::<Point2D>::new();
    let points = vec![
        Point2D { x: 0, y: 0 },
        Point2D { x: 10, y: 10 },
        Point2D { x: -10, y: -10 },
        Point2D { x: 5, y: 5 },
    ];
    for p in &points {
        tree.insert(p.clone());
    }

    let to_remove = Point2D { x: 5, y: 5 };
    tree.remove(&to_remove);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!("Cover tree validation failed after removal: {}", e);
        tree.print();
        panic!();
    }

    let query = Point2D { x: 5, y: 5 };
    let result = tree.nearest_neighbor(&query);
    assert!(result.is_some());
    let (nn, _) = result.unwrap();
    assert_ne!(nn, to_remove);
}

#[test]
fn test_empty_tree_nearest_neighbor_is_none() {
    let tree = CoverTree::<Point2D>::new();
    let query = Point2D { x: 1, y: 2 };
    assert!(tree.nearest_neighbor(&query).is_none());
}

#[test]
fn test_remove_root_child_and_validate() {
    let mut tree = CoverTree::<Point2D>::new();
    let p1 = Point2D { x: 0, y: 0 };
    let p2 = Point2D { x: 1, y: 1 };

    tree.insert(p1.clone());
    tree.insert(p2.clone());

    tree.remove(&p2);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!(
            "Cover tree validation failed after removing root child: {}",
            e
        );
        tree.print();
        panic!();
    }
}

#[test]
fn test_insert_many_points_and_validate_tree() {
    let mut tree = CoverTree::<Point2D>::new();
    tree.print();
    for x in -20..=20 {
        for y in -20..=20 {
            if x % 10 == 0 && y % 10 == 0 {
                tree.insert(Point2D { x, y });
                tree.print();
                if let Err(e) = tree.assert_valid_cover_tree() {
                    println!(
                        "Cover tree validation failed after inserting point ({}, {}): {}",
                        x, y, e
                    );
                    tree.print();
                    panic!();
                }
            }
        }
    }
}

#[test]
fn test_print_tree() {
    let mut tree = CoverTree::<Point2D>::new();
    tree.insert(Point2D { x: 0, y: 0 });
    tree.insert(Point2D { x: 5, y: 0 });
    tree.insert(Point2D { x: 10, y: 10 });
    tree.print();
}

#[test]
fn test_remove_root() {
    let mut tree = CoverTree::<Point2D>::new();
    let points = vec![Point2D { x: 0, y: 0 }, Point2D { x: 10, y: 10 }];
    for p in &points {
        tree.insert(p.clone());
    }
    tree.print();
    tree.remove(&points[0]);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!("Cover tree validation failed after removing root: {}", e);
        tree.print();
        panic!();
    }
    tree.remove(&points[1]);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!(
            "Cover tree validation failed after removing last node: {}",
            e
        );
        tree.print();
        panic!();
    }
}

#[test]
fn test_remove_root3() {
    let mut tree = CoverTree::<Point2D>::new();
    let points = vec![
        Point2D { x: 0, y: 0 },
        Point2D { x: 20, y: 20 },
        Point2D { x: 10, y: 10 },
    ];
    for p in &points {
        tree.insert(p.clone());
    }
    tree.print();
    println!("Removing root node");
    tree.remove(&points[0]);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!("Cover tree validation failed after removing root: {}", e);
        tree.print();
        panic!();
    }
    tree.print();
    println!("Removing second node");
    tree.remove(&points[1]);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!(
            "Cover tree validation failed after removing second node: {}",
            e
        );
        tree.print();
        panic!();
    }
    tree.print();
    println!("Removing last node");
    tree.remove(&points[2]);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!(
            "Cover tree validation failed after removing last node: {}",
            e
        );
        tree.print();
        panic!();
    }
}

#[test]
fn test_remove_only_root() {
    let mut tree = CoverTree::<Point2D>::new();
    let p = Point2D { x: 0, y: 0 };
    tree.insert(p.clone());
    tree.print();
    tree.remove(&p);
    if let Err(e) = tree.assert_valid_cover_tree() {
        println!("Cover tree validation failed after removing root: {}", e);
        tree.print();
        panic!();
    }
    tree.print();
}
