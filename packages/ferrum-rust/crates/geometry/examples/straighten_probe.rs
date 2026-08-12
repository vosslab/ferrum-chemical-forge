//! JSON probe for comparing Ferrum's arithmetic port with a separate RDKit process.

use ferrum_geometry::{Point2, straighten_depiction};

fn point(x: f64, y: f64) -> Point2 {
    Point2::new(x, y).expect("probe input is finite")
}

fn main() {
    let cases = [
        (
            "ten_degree_bond",
            vec![
                point(0.0, 0.0),
                point(10_f64.to_radians().cos(), 10_f64.to_radians().sin()),
            ],
            vec![(0, 1)],
        ),
        (
            "fifteen_degree_boundary",
            vec![
                point(0.0, 0.0),
                point(15_f64.to_radians().cos(), 15_f64.to_radians().sin()),
            ],
            vec![(0, 1)],
        ),
        (
            "thirty_degree_boundary",
            vec![
                point(0.0, 0.0),
                point(30_f64.to_radians().cos(), 30_f64.to_radians().sin()),
            ],
            vec![(0, 1)],
        ),
        (
            "asymmetric_three_bond",
            vec![
                point(0.0, 0.0),
                point(1.0, 0.2),
                point(1.7, 1.1),
                point(2.4, 1.35),
            ],
            vec![(0, 1), (1, 2), (2, 3)],
        ),
    ];
    print!("{{\"cases\":[");
    for (case_index, (name, points, bonds)) in cases.iter().enumerate() {
        if case_index > 0 {
            print!(",");
        }
        print!("{{\"name\":\"{name}\",\"branches\":{{");
        for (branch_index, minimize_rotation) in [false, true].iter().enumerate() {
            if branch_index > 0 {
                print!(",");
            }
            let result = straighten_depiction(points, bonds, *minimize_rotation)
                .expect("probe bond indices are valid");
            print!(
                "\"{minimize_rotation}\":{{\"rotation_radians\":{:.17},\"coordinates\":[",
                result.rotation_radians
            );
            for (point_index, coordinate) in result.coordinates.iter().enumerate() {
                if point_index > 0 {
                    print!(",");
                }
                print!("[{:.17},{:.17}]", coordinate.x(), coordinate.y());
            }
            print!("]}}");
        }
        print!("}}}}");
    }
    println!("]}}");
}
