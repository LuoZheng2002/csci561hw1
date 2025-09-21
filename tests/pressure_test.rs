use rust::cover_tree::CoverTree;
use rust::generator::ProblemGenerator;

#[test]
fn pressure_test_cover_tree_nearest_neighbor() {
    let num_points = 20;
    let max_radius = 10.0;
    let num_queries = 1;

    let problem_gen = ProblemGenerator::new(num_points, max_radius);
    let problem = problem_gen.generate_problem();
    let cities = problem.cities;

    let mut tree = CoverTree::new();

    for city in &cities {
        tree.insert(city.clone());
    }

    tree.print();

    println!("Finished building the cover tree.");

    // Validate structure (optional)
    if let Err(e) = tree.assert_valid_cover_tree() {
        panic!("Cover tree failed validation: {}", e);
    }

    let mut num_mismatches = 0;

    for (i, query) in cities.iter().take(num_queries).enumerate() {
        let brute_result = cities
            .iter()
            .filter(|p| *p != query)
            .map(|p| (p, p.distance(query)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        tree.remove(query);
        let tree_result = tree.nearest_neighbor(query).unwrap();

        let dist_diff = (tree_result.1 - brute_result.1).abs();
        let same_point = &tree_result.0 == brute_result.0;

        if dist_diff > 1e-4 || !same_point {
            println!(
                "Mismatch at query {}: brute {:?} vs tree {:?}, dist diff {}",
                i, brute_result, tree_result, dist_diff
            );
            num_mismatches += 1;
        }
    }

    assert_eq!(num_mismatches, 0, "Found {} mismatches!", num_mismatches);
}
