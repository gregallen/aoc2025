use crate::Direction::North;
use crate::Outline::{Horizontal, Vertical};
use Direction::{East, South, West};
use anyhow::Error;
use derivative::Derivative;
use simple_svg::Group;
use simple_svg::Polyline;
use simple_svg::Shape;
use simple_svg::Sstyle;
use simple_svg::Svg;
use simple_svg::Widget;
use simple_svg::svg_out;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};
use std::ops::RangeInclusive;

#[derive(Debug, PartialEq, Clone, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

/// SVG style: `y` increases downwards, `x` increases rightwards. South is y increasing, East is x increasing.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Outline {
    Horizontal(Direction, RangeInclusive<usize>),
    Vertical(Direction, usize),
}

impl Outline {
    #[inline]
    fn index(&self) -> usize {
        match self {
            Horizontal(_, h) => *h.start(),
            Vertical(_, v) => *v,
        }
    }
}

impl Ord for Outline {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.index().cmp(&other.index())
    }
}
impl PartialOrd for Outline {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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

    // let area = area(text)?;
    // println!("area: {area}");

    let area = tiled_area(text)?;
    println!("area: {area}");

    Ok(())
}

#[derive(Derivative, Clone, Debug, Hash, PartialEq, Eq)]
struct Point {
    x: usize,
    y: usize,
}

fn _area(lines: Vec<String>) -> anyhow::Result<usize> {
    let points = parse_lines(lines);
    let num_points = points.len();
    let pairs = points.iter().enumerate().map(|(i, p)| {
        points[i + 1..num_points]
            .iter()
            .map(|q| weight(p, q))
            .max()
            .unwrap_or(0)
    });

    let largest_area = pairs.max().unwrap_or(0);
    Ok(largest_area)
}

fn tiled_area(lines: Vec<String>) -> anyhow::Result<usize> {
    let points = parse_lines(lines);

    let max_x = points.iter().map(|p| p.x).max().unwrap_or(0);
    let max_y = points.iter().map(|p| p.y).max().unwrap_or(0);
    to_svg("day9.svg", &points, max_x, max_y);

    let mut bitmap = vec![BTreeSet::new(); max_y + 1];
    draw(&points, &mut bitmap);

    let filled_rows = bitmap.iter().map(|r| fill_row(r)).collect::<Vec<_>>();

    let num_points = points.len();
    let mut all_pairs = (0..num_points)
        .flat_map(move |i| ((i + 1)..num_points).map(move |j| (i, j)))
        .collect::<Vec<_>>();
    all_pairs.sort_by_cached_key(|&(i, j)| weight(&points[i], &points[j]));
    eprintln!("Sorted {} pairs by weight", all_pairs.len());

    let largest_corners = all_pairs
        .iter()
        .rev()
        .find(|&(i, j)| is_covered(&points[*i], &points[*j], &filled_rows))
        .expect("No covered rectangles found!");

    let largest_area = weight(&points[largest_corners.0], &points[largest_corners.1]);
    Ok(largest_area)
}

fn is_covered(p: &Point, q: &Point, bitmap: &Vec<Vec<RangeInclusive<usize>>>) -> bool {
    let min_x = p.x.min(q.x);
    let min_y = p.y.min(q.y);

    let max_x = p.x.max(q.x);
    let max_y = p.y.max(q.y);

    // For each horizontal strip of (p,q), find the range that includes the first point
    // and check that the same range encloses the second point.
    let counter_example = bitmap[min_y..=max_y].iter().find(|row| {
        !row.iter()
            .find(|r| r.contains(&min_x))
            .map(|r| r.contains(&max_x))
            .unwrap_or(false)
    });

    counter_example.is_none()
}

/// south is y increasing
/// east is x increasing
fn direction(from: &Point, to: &Point) -> Direction {
    if from.x == to.x {
        // north or south
        if from.y < to.y { South } else { North }
    } else {
        if from.x < to.x { East } else { West }
    }
}

fn draw(points: &[Point], bitmap: &mut Vec<BTreeSet<Outline>>) {
    let num_points = points.len();

    for (i, point) in points.iter().enumerate() {
        let Point { x, y } = point;
        let prev = &points[(i + num_points - 1) % num_points];

        let direction = direction(prev, &point);

        let outline = match direction {
            North => Vertical(North, *x),
            South => Vertical(South, *x),
            East => Horizontal(East, prev.x..=*x),
            West => Horizontal(West, *x..=prev.x),
        };

        match direction {
            North | South => {
                let from_y = prev.y.min(*y) + 1;
                let to_y = prev.y.max(*y);
                for bits in &mut bitmap[from_y..to_y] {
                    bits.insert(outline.clone());
                }
            }
            East | West => {
                bitmap[*y].insert(outline);
            }
        }
    }
}

/// take an outline and replaces the edge with simple contiguous horizontal ranges
/// Fill is assumed to be on the right of the line - i.e. line direction is clockwise around enclosed space
/// i.e. fill is always to the +x of North and -x of South
fn fill_row(row: &BTreeSet<Outline>) -> Vec<RangeInclusive<usize>> {
    let mut new_row = vec![];

    let mut outline_iter = row.iter().peekable();
    while let Some(o) = outline_iter.next() {
        match o {
            Vertical(North, left) => {
                let mut rightmost = left;
                while let Some(next) = outline_iter.peek() {
                    match next {
                        Horizontal(_, rang) => {
                            rightmost = rang.end();
                            outline_iter.next();
                        }
                        Vertical(South, right) => {
                            rightmost = right;
                            outline_iter.next();
                            break;
                        }
                        Vertical(North, _) => {
                            break;
                        }
                        _ => {
                            panic!("Impossible outline after North: {:?}", next);
                        }
                    }
                }
                new_row.push(*left..=*rightmost);
            }
            Horizontal(_, first_rang) => {
                let mut rightmost = first_rang.end();
                while let Some(next) = outline_iter.peek() {
                    match next {
                        Horizontal(_, _) => {
                            break;
                        }
                        Vertical(South, right) => {
                            rightmost = right;
                            outline_iter.next();
                            break;
                        }
                        Vertical(North, _) => {
                            break;
                        }
                        _ => {
                            panic!("Impossible outline after {o:?}: {next:?}");
                        }
                    }
                }
                new_row.push(*first_rang.start()..=*rightmost);
            }
            Vertical(_, _) => {
                panic!("Unexpected {o:?} at start of row {row:?}")
            }
        }
    }
    new_row
}

fn to_svg(file: &str, points: &Vec<Point>, width: usize, height: usize) {
    let mut svg = Svg::new(width as f64, height as f64);

    let mut polyline_sstyle = Sstyle::new();
    polyline_sstyle.fill = Some("black".to_string());
    polyline_sstyle.stroke = Some("white".to_string());
    polyline_sstyle.stroke_width = Some(25.0);

    let polyline_id = svg.add_shape(Shape::Polyline(Polyline::new(
        points.iter().map(|p| (p.x as f64, p.y as f64)).collect(),
    )));

    let mut group = Group::new();
    group.place_widget(Widget {
        shape_id: polyline_id,
        style: Some(polyline_sstyle),
        at: Some((0.0, 0.0)),
        ..Default::default()
    });

    svg.add_default_group(group);

    let svg_str = svg_out(svg);

    std::fs::write(file, svg_str).expect("Unable to write to file");
}

fn parse_lines(lines: Vec<String>) -> Vec<Point> {
    lines
        .iter()
        .map(|l| {
            let mut parts = l.split(',').map(|d| d.parse::<usize>().unwrap());
            Point {
                x: parts.next().unwrap(),
                y: parts.next().unwrap(),
            }
        })
        .collect()
}

#[inline]
fn weight(a: &Point, b: &Point) -> usize {
    (a.x.max(b.x) - a.x.min(b.x) + 1) * (a.y.max(b.y) - a.y.min(b.y) + 1)
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
        let result = _area(lines);
        assert_eq!(50, result.unwrap());
    }

    #[test]
    fn part2() {
        let lines = EXAMPLE_INPUT.split('\n').map(String::from).collect();
        let result = tiled_area(lines);
        assert_eq!(24, result.unwrap());
    }

    #[test]
    fn draw_clockwise_square() {
        let lines = r"0,0
2,0
2,2
0,2"
        .split('\n')
        .map(String::from)
        .collect();
        let points = parse_lines(lines);
        let mut bitmap = vec![BTreeSet::new(); 3];
        draw(&points, &mut bitmap);

        assert_eq!(
            vec![
                BTreeSet::from([Horizontal(East, 0..=2)]),
                BTreeSet::from([Vertical(North, 0), Vertical(South, 2)]),
                BTreeSet::from([Horizontal(West, 0..=2)]),
            ],
            bitmap
        );
    }

    #[test]
    fn draw_anti_clockwise_square() {
        let lines = r"0,0
0,2
2,2
2,0"
        .split('\n')
        .map(String::from)
        .collect();
        let points = parse_lines(lines);
        let mut bitmap = vec![BTreeSet::new(); 3];
        draw(&points, &mut bitmap);

        assert_eq!(
            vec![
                BTreeSet::from([Horizontal(West, 0..=2)]),
                BTreeSet::from([Vertical(South, 0), Vertical(North, 2)]),
                BTreeSet::from([Horizontal(East, 0..=2)]),
            ],
            bitmap
        );
    }

    #[test]
    fn test_fill_row() {
        assert_eq!(
            vec![1..=4, 6..=7],
            fill_row(&BTreeSet::from([
                Horizontal(East, 1..=2),
                Vertical(South, 4),
                Vertical(North, 6),
                Vertical(South, 7),
            ]))
        );
    }

    #[test]
    fn test_is_covered_single_line() {
        assert!(!is_covered(
            &Point { x: 0, y: 0 },
            &Point { x: 0, y: 0 },
            &vec![vec![]]
        ));
        assert!(!is_covered(
            &Point { x: 0, y: 0 },
            &Point { x: 1, y: 0 },
            &vec![vec![]]
        ));
        assert!(!is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 0, y: 0 },
            &vec![vec![]]
        ));

        assert!(!is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 0, y: 0 },
            &vec![vec![1..=2]]
        ));
        assert!(!is_covered(
            &Point { x: 0, y: 0 },
            &Point { x: 2, y: 0 },
            &vec![vec![1..=2]]
        ));
        assert!(!is_covered(
            &Point { x: 0, y: 0 },
            &Point { x: 3, y: 0 },
            &vec![vec![1..=2]]
        ));

        assert!(is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 1, y: 0 },
            &vec![vec![1..=2]]
        ));
        assert!(is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 2, y: 0 },
            &vec![vec![1..=2]]
        ));
        assert!(is_covered(
            &Point { x: 2, y: 0 },
            &Point { x: 2, y: 0 },
            &vec![vec![1..=2]]
        ));

        assert!(is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 3, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 4, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 5, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 6, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(!is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 7, y: 0 },
            &vec![vec![1..=6]]
        ));

        assert!(is_covered(
            &Point { x: 4, y: 0 },
            &Point { x: 4, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(is_covered(
            &Point { x: 5, y: 0 },
            &Point { x: 5, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(is_covered(
            &Point { x: 6, y: 0 },
            &Point { x: 6, y: 0 },
            &vec![vec![1..=6]]
        ));

        assert!(!is_covered(
            &Point { x: 3, y: 0 },
            &Point { x: 7, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(!is_covered(
            &Point { x: 6, y: 0 },
            &Point { x: 7, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(!is_covered(
            &Point { x: 4, y: 0 },
            &Point { x: 7, y: 0 },
            &vec![vec![1..=6]]
        ));
    }

    #[test]
    fn test_draw_example() {
        let lines = EXAMPLE_INPUT.split('\n').map(String::from).collect();
        let points = parse_lines(lines);

        let max_y = points.iter().map(|p| p.y).max().unwrap_or(0);

        let mut bitmap = vec![BTreeSet::new(); max_y + 1];
        draw(&points, &mut bitmap);

        bitmap.iter().for_each(|row| println!("{row:?}"));

        let rows = bitmap.iter().map(fill_row).collect::<Vec<_>>();

        // nasty edge case on row 3
        assert!(is_covered(
            &Point { x: 7, y: 3 },
            &Point { x: 7, y: 3 },
            &rows
        ));
        // still have an odd number of verticals to the right, so all good
        assert!(is_covered(
            &Point { x: 7, y: 3 },
            &Point { x: 8, y: 3 },
            &rows
        ));
        assert!(is_covered(
            &Point { x: 7, y: 3 },
            &Point { x: 11, y: 3 },
            &rows
        ));
        assert!(is_covered(
            &Point { x: 2, y: 3 },
            &Point { x: 9, y: 3 },
            &rows
        ));
        assert!(is_covered(
            &Point { x: 2, y: 5 },
            &Point { x: 9, y: 5 },
            &rows
        ));
        assert!(is_covered(
            &Point { x: 2, y: 4 },
            &Point { x: 9, y: 4 },
            &rows
        ));
        assert!(is_covered(
            &Point { x: 2, y: 3 },
            &Point { x: 9, y: 5 },
            &rows
        ));
    }

    #[test]
    fn test_fill() {
        let mut bitmap = vec![BTreeSet::new(); 5];
        let points = vec![
            Point { x: 1, y: 1 },
            Point { x: 3, y: 1 },
            Point { x: 3, y: 3 },
            Point { x: 1, y: 3 },
        ];
        draw(&points, &mut bitmap);

        assert_eq!(
            vec![
                BTreeSet::new(),
                BTreeSet::from([Horizontal(East, 1..=3)]),
                BTreeSet::from([Vertical(North, 1), Vertical(South, 3)]),
                BTreeSet::from([Horizontal(West, 1..=3)]),
                BTreeSet::new(),
            ],
            bitmap
        );

        to_svg("day9_test.svg", &points, 5, 5);

        for bits in &bitmap {
            println!("{bits:?}");
        }
    }

    #[test]
    fn test_fill_row2() {
        let row: Vec<RangeInclusive<usize>> = fill_row(&BTreeSet::from([]));
        assert_eq!(Vec::<RangeInclusive<usize>>::new(), row);

        assert_eq!(
            vec![1..=3],
            fill_row(&BTreeSet::from([Horizontal(East, 1..=3)]))
        );

        assert_eq!(
            vec![1..=3, 10..=20],
            fill_row(&BTreeSet::from([
                Horizontal(East, 1..=3),
                Horizontal(East, 10..=20)
            ]))
        );

        assert_eq!(
            vec![1..=3],
            fill_row(&BTreeSet::from([Vertical(North, 1), Vertical(South, 3)]))
        );

        assert_eq!(
            vec![1..=3, 10..=20],
            fill_row(&BTreeSet::from([
                Vertical(North, 1),
                Vertical(South, 3),
                Vertical(North, 10),
                Vertical(South, 20),
            ]))
        );

        assert_eq!(
            vec![1..=20],
            fill_row(&BTreeSet::from([
                Vertical(North, 1),
                Horizontal(East, 10..=20)
            ]))
        );
    }
}
