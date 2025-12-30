use anyhow::Error;
use derivative::Derivative;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};
use std::{env, mem};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() < 2 {
        "src/bin/day9.txt"
    } else {
        &args[1]
    };

    let file: Box<dyn BufRead> = if filename == "-" {
        Box::new(stdin().lock())
    } else {
        Box::new(BufReader::new(
            File::open(filename).expect("Failed to open file"),
        ))
    };
    let text = file.lines().map(Result::unwrap).collect::<Vec<_>>();

    let result = junctions(text, 1_000_000)?;

    println!("{result}");

    Ok(())
}

#[derive(Derivative, Clone, Debug)]
#[derivative(Hash, PartialEq, Eq)]
struct Point {
    x: i64,
    y: i64,
    z: i64,
    #[derivative(Hash = "ignore")]
    #[derivative(PartialEq = "ignore")]
    id: usize,
}

fn junctions(lines: Vec<String>, connections: usize) -> anyhow::Result<usize> {
    let points = parse_lines(lines);
    let mut circuits = (0..points.len())
        .map(|i| HashSet::from([i]))
        .collect::<Vec<_>>();
    let mut point_to_circuit = (0..points.len()).collect::<Vec<usize>>();
    let mut junctions: HashSet<(usize, usize)> = HashSet::new();

    for _ in 0..connections {
        let (left_point_id, right_point_id) = closest_neighbours(&points, &circuits, &junctions);

        junctions.insert((left_point_id, right_point_id));
        junctions.insert((right_point_id, left_point_id));

        let left_circuit_id = point_to_circuit[left_point_id];
        let right_circuit_id = point_to_circuit[right_point_id];
        if left_circuit_id != right_circuit_id {
            let mut obsolete_circuit_point_ids = mem::take(&mut circuits[right_circuit_id]);
            for point_id in obsolete_circuit_point_ids.iter() {
                point_to_circuit[*point_id] = left_circuit_id;
            }
            circuits[left_circuit_id].extend(obsolete_circuit_point_ids.drain());

            // are all the points now in one circuit?
            if circuits[left_circuit_id].len() == points.len() {
                let result = points[left_point_id].x as usize * points[right_point_id].x as usize;
                println!("*** {result} !!!");
                return Ok(result);
            }
        };
    }
    // find 3 largest circuits
    let mut circuit_sizes = circuits.iter().map(HashSet::len).collect::<Vec<_>>();
    circuit_sizes.sort();
    println!("{circuit_sizes:?}");
    Ok(circuit_sizes.into_iter().rev().take(3).product())
}

fn parse_lines(lines: Vec<String>) -> Vec<Point> {
    lines
        .iter()
        .enumerate()
        .map(|(id, l)| {
            let mut parts = l.split(',').map(|d| d.parse::<i64>().unwrap());
            Point {
                x: parts.next().unwrap(),
                y: parts.next().unwrap(),
                z: parts.next().unwrap(),
                id,
            }
        })
        .collect()
}

fn closest_neighbours(
    points: &[Point],
    circuits: &[HashSet<usize>],
    junctions: &HashSet<(usize, usize)>,
) -> (usize, usize) {
    let midpoints: Vec<_> = circuits
        .iter()
        .filter(|circuit| !circuit.is_empty())
        .map(|point_ids| midpoint(points, point_ids))
        .collect();
    let num_midpoints = midpoints.len();

    let midpoint_min_distance2_estimate = midpoints
        .iter()
        .enumerate()
        .map(|(i, p)| (p, &midpoints[(i + 1) % num_midpoints]))
        .min_by_key(tuple_weight)
        .as_ref()
        .map(tuple_weight)
        .unwrap_or(1);
    let midpoint_min_distance_estimate =
        (midpoint_min_distance2_estimate as f64).sqrt().round() as i64;
    println!("midpoint distance estimate: {midpoint_min_distance_estimate}");

    let mut blocks: HashMap<(i64, i64, i64), Vec<&Point>> = HashMap::new();

    for point in points {
        let xb = point.x / midpoint_min_distance_estimate;
        let yb = point.y / midpoint_min_distance_estimate;
        let zb = point.z / midpoint_min_distance_estimate;
        blocks.entry((xb, yb, zb)).or_default().push(point);
    }

    // for all blocks with at least 2 points, compute min pair
    let closest_in_block = blocks
        .iter()
        .filter(|(_, points)| points.len() >= 2)
        .filter_map(|(_, points)| {
            points
                .iter()
                .enumerate()
                .flat_map(|(i, p)| {
                    points
                        .iter()
                        .skip(i + 1)
                        .filter(|q| !junctions.contains(&(p.id, q.id)))
                        // .filter(|q| point_to_circuit[p.id] != point_to_circuit[q.id])
                        .map(move |q| (*p, *q))
                })
                .min_by_key(tuple_weight)
        });

    let neighbours: Vec<_> = (0..=1)
        .flat_map(|dx| {
            (-1..=1).flat_map(move |dy| {
                (-1..=1)
                    .filter(move |dz| dx != 0 || dy != 0 || *dz != 0)
                    .map(move |dz| (dx, dy, dz))
            })
        })
        .collect();

    let result = blocks
        .iter()
        .flat_map(|((bx, by, bz), points)| {
            neighbours
                .iter()
                .map(move |offset| (bx + offset.0, by + offset.1, bz + offset.2))
                .filter_map(|c| blocks.get(&c))
                .flat_map(|others| {
                    points.iter().flat_map(|p| {
                        others
                            .iter()
                            .filter(|q| !junctions.contains(&(p.id, q.id)))
                            .map(move |q| (*p, *q))
                    })
                })
                .min_by_key(tuple_weight)
        })
        .chain(closest_in_block)
        .min_by_key(tuple_weight)
        .expect("No neighbours!");

    println!(
        "joining {result:?}, points {} & {}",
        result.0.id, result.1.id,
    );

    (result.0.id, result.1.id)
}

fn midpoint<'a, I>(points: &[Point], point_ids: I) -> Point
where
    I: IntoIterator<Item = &'a usize>,
{
    if points.len() == 1 {
        points[0].clone()
    } else {
        let mut count = 0;
        let mut mean = Point {
            x: 0,
            y: 0,
            z: 0,
            id: 0,
        };
        for point_id in point_ids {
            let point = &points[*point_id];
            count += 1;
            mean.x += point.x;
            mean.y += point.y;
            mean.z += point.z
        }
        mean.x /= count;
        mean.y /= count;
        mean.z /= count;
        mean
    }
}

#[inline]
fn tuple_weight((a, b): &(&Point, &Point)) -> i64 {
    weight(a, b)
}

#[inline]
fn weight(a: &Point, b: &Point) -> i64 {
    (a.x - b.x).pow(2) + (a.y - b.y).pow(2) + (a.z - b.z).pow(2)
}

#[cfg(test)]
mod tests {
    use crate::*;

    const EXAMPLE_INPUT: &str = r"7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3";

    #[test]
    fn example() {
        let lines = EXAMPLE_INPUT.split('\n').map(String::from).collect();
        let result = junctions(lines, 10);
        assert_eq!(40, result.unwrap());
    }

    #[test]
    fn part2() {
        let lines = EXAMPLE_INPUT.split('\n').map(String::from).collect();
        let result = junctions(lines, 10000);
        assert_eq!(25272, result.unwrap());
    }

    #[test]
    fn closest_neighbours_example() {
        let points = example_points();

        let mut circuits = (0..points.len())
            .map(|i| HashSet::from([i]))
            .collect::<Vec<_>>();
        let mut point_to_circuit = (0..points.len()).collect::<Vec<usize>>();
        let mut junctions: HashSet<(usize, usize)> = HashSet::new();

        let result = closest_neighbours(&points, &circuits, &junctions);
        assert!(matches!(result, (0, 19) | (19, 0)));
        circuits[0].insert(19);
        circuits[19].clear();
        point_to_circuit[19] = 0;
        junctions.insert((0, 19));
        junctions.insert((19, 0));

        let result2 = closest_neighbours(&points, &circuits, &junctions);
        assert!(matches!(result2, (0, 7) | (7, 0)));
        circuits[0].insert(7);
        circuits[7].clear();
        point_to_circuit[7] = 0;
        junctions.insert((0, 7));
        junctions.insert((7, 0));

        let results3 = closest_neighbours(&points, &circuits, &junctions);
        assert!(matches!(results3, (2, 13) | (13, 2)));
    }

    fn example_points() -> Vec<Point> {
        let lines = EXAMPLE_INPUT.split('\n').map(String::from).collect();
        parse_lines(lines)
    }
}
