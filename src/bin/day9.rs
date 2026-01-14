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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Turn {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn which_turn(&self, previously: &Direction) -> Turn {
        match (previously, self) {
            (North, East) => Turn::Right,
            (East, South) => Turn::Right,
            (South, West) => Turn::Right,
            (West, North) => Turn::Right,

            (North, West) => Turn::Left,
            (West, South) => Turn::Left,
            (South, East) => Turn::Left,
            (East, North) => Turn::Left,

            (_, _) => {
                panic!("Can't turn from {previously:?} to {self:?}")
            }
        }
    }

    fn turn(&self, turn: Turn) -> Direction {
        match turn {
            Turn::Left => match self {
                North => West,
                East => North,
                South => East,
                West => South,
            },
            Turn::Right => match self {
                North => East,
                East => South,
                South => West,
                West => North,
            },
        }
    }
}

/// SVG style: `y` increases downwards, `x` increases rightwards. South is y increasing, East is x increasing.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Outline {
    Horizontal(Direction, RangeInclusive<usize>, Turn),
    /// increasing x
    /// decreasing y
    Vertical(Direction, usize, Turn),
}

impl Outline {
    #[inline]
    fn index(&self) -> usize {
        match self {
            Horizontal(_, h, _) => *h.start(),
            Vertical(_, v, _) => *v,
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

fn area(lines: Vec<String>) -> anyhow::Result<usize> {
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
    let num_points = points.len();

    let mut bitmap = vec![BTreeSet::new(); max_y + 1];
    draw(&points, &mut bitmap);

    let filled_rows = bitmap.iter().map(|r| fill_row(r)).collect::<Vec<_>>();

    // show lines 1774..=1776
    for i in 1774..=1776 {
        println!("row {i} {:?}", bitmap[i]);
    }

    // to_png("day9.png", &bitmap, max_x as u32 + 1, max_y as u32 + 1);

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
    // let min_x = p.x.min(q.x);
    let min_y = p.y.min(q.y);

    // let max_x = p.x.max(q.x);
    let max_y = p.y.max(q.y);

    // For each horizontal strip of (p,q), walk from left edge and check that an odd number of vertical Outlines are crossed,
    // or the point lies on a vertical or wholly inside a horizontal.
    // If we find any strip of the rectangle starts after an even number of boundaries
    // we can immediately return false
    // let left_edge = Vertical(Northward, min_x);
    // let right_edge = Vertical(Northward, max_x);
    // let past_right_edge = Vertical(Northward, max_x + 1);
    let counter_example = bitmap[min_y..=max_y].iter().find(|row| {
        // let to_the_left = row.range(..=&left_edge);
        // let verticals_to_left: usize = to_the_left.clone().map(Outline::verticals).sum();
        // if let Some(outline_at_left_edge) = to_the_left.last() {
        //     match outline_at_left_edge {
        //         Horizontal(rang) => {
        //             if !rang.contains(&min_x) {
        //                 return true;
        //             }
        //         }
        //         Vertical(v) => {
        //             if verticals_to_left % 2 == 0 {
        //                 // immediately to right must be untiled, will be a counter example unless
        //                 // we're width 1 and right on the edge. i.e. min_x == max_x == vertical?
        //                 return *v != min_x || min_x != max_x;
        //             }
        //         }
        //     }
        //     if min_x == max_x {
        //         return false;
        //     }
        //     // walking to right-edge must not cross any verticals (but can cross horizontal)
        //     // excepting case where we finish exactly one & there are even verticals to the right
        //     let vertical_crossed = row
        //         .range(&left_edge..&right_edge)
        //         .find(|o| matches!(o, Vertical(v) if *v > min_x))
        //         .is_some();
        //     if vertical_crossed {
        //         return true;
        //     }
        //
        //     // finally, max_x must be inside preceding outline or have odd number of verticals to right
        //     // or be exactly on vertical with even verticals to right
        //     let even_verticals_to_past_right = row
        //         .range(&past_right_edge..)
        //         .map(Outline::verticals)
        //         .sum::<usize>()
        //         % 2
        //         == 0;
        //     if !even_verticals_to_past_right {
        //         // if right edge actually falls right on the start of next region that's a counter example
        //         return row.contains(&right_edge);
        //     }
        //     let outline_to_left_of_right_edge = row
        //         .range(..=&right_edge)
        //         .last()
        //         .expect("No outline before right edge, but there was one for left edge!");
        //     match outline_to_left_of_right_edge {
        //         Horizontal(rang) => !rang.contains(&max_x) && even_verticals_to_past_right,
        //         Vertical(v) => *v != max_x || !even_verticals_to_past_right,
        //     }
        // } else {
        // left edge is untiled
        return true;
        // }
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
    let mut previous_direction = direction(&points[num_points - 2], &points[num_points - 1]);

    for (i, point) in points.iter().enumerate() {
        let Point { x, y } = point;
        let prev = &points[(i + num_points - 1) % num_points];

        let direction = direction(prev, &point);
        let turn = direction.which_turn(&previous_direction);

        let outline = match direction {
            North => Vertical(North, *x, turn),
            South => Vertical(South, *x, turn),
            East => Horizontal(East, prev.x..=*x, turn),
            West => Horizontal(West, *x..=prev.x, turn),
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

        previous_direction = direction;
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
            Vertical(North, left, _) => {
                let mut rightmost = None;
                // this is typical at left edge
                // fill through any WR, terminating at a WL or N?, or just this outline if next is none or S?
                while let Some(next) = outline_iter.peek() {
                    match next {
                        Horizontal(West, rang, Turn::Right) => {
                            outline_iter.next();
                            rightmost = Some(rang.end());
                        }
                        Horizontal(West, rang, Turn::Left) => {
                            // in this case we stop filling
                            outline_iter.next();
                            rightmost = Some(rang.end());
                            break;
                        }
                        Vertical(North, right, _) => {
                            rightmost = Some(right);
                        }
                        _ => {
                            panic!("Impossible outline after South: {:?}", next);
                        }
                    }
                }
                new_row.push(*left..=*rightmost.expect("No right edge!"));
            }
            Horizontal(West, rang, Turn::Left) => {
                // typically around 12 o'clock position
                // ??? Sounds wrong. surely always going East/right here?
                panic!("Unexpected {o:?} at start of row {row:?}")
                // new_row.push(rang.clone());
            }
            Horizontal(direction, rang, Turn::Right) => {
                // likely a "peak" around 6 o'clock position, or low hanging peak around 12 o'clock position
                // but if we see a South after east, or North after West it was actually a trough
                // TODO implement this!
                let mut rightmost = rang.end();
                while let Some(next) = outline_iter.peek() {
                    match next {
                        Horizontal(West, rang, Turn::Right) => {
                            outline_iter.next();
                            rightmost = rang.end();
                        }
                        Horizontal(West, rang, Turn::Left) => {
                            // in this case we stop filling
                            outline_iter.next();
                            rightmost = rang.end();
                            break;
                        }
                        Vertical(North, right, _) => {
                            rightmost = right;
                        }
                        _ => {
                            panic!("Impossible outline after South: {:?}", next);
                        }
                    }
                }
                new_row.push(*rang.start()..=*rightmost);
            }
            Horizontal(direction, rang, Turn::Left) => {
                // typically around 6 o'clock position
                new_row.push(rang.clone());
            }
            Vertical(_, _, _) => {
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

    eprintln!("Drawing image...");

    let mut group = Group::new();
    group.place_widget(Widget {
        shape_id: polyline_id,
        style: Some(polyline_sstyle),
        at: Some((0.0, 0.0)),
        ..Default::default()
    });

    svg.add_default_group(group);

    let svg_str = svg_out(svg);
    eprintln!("Done drawing image.");

    // write svg_str to file
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
    use crate::Turn::{Left, Right};
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
        let result = area(lines);
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
                BTreeSet::from([Horizontal(East, 0..=2, Right)]),
                BTreeSet::from([Vertical(North, 0, Right), Vertical(South, 2, Right)]),
                BTreeSet::from([Horizontal(West, 0..=2, Right)]),
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
                BTreeSet::from([Horizontal(West, 0..=2, Left)]),
                BTreeSet::from([Vertical(South, 0, Left), Vertical(North, 2, Left)]),
                BTreeSet::from([Horizontal(East, 0..=2, Left)]),
            ],
            bitmap
        );
    }

    #[test]
    fn test_fill_row() {
        assert_eq!(
            vec![1..=6],
            fill_row(&BTreeSet::from([
                Horizontal(East, 1..=2, Right),
                Vertical(South, 4, Right),
                Vertical(North, 6, Left)
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

        assert!(!is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 3, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(!is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 4, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(!is_covered(
            &Point { x: 1, y: 0 },
            &Point { x: 5, y: 0 },
            &vec![vec![1..=6]]
        ));
        assert!(!is_covered(
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

        // its fine to span multiple horizontal lines, skimming a horizontal edge, but can't cross a vertical
        // THIS CASE NOT ALLOWED, SINCE CORNERS MUST LIE ON HORIZONTALS OR VERTICALS
        // assert!(!is_covered(
        //     &Point { x: 7, y: 3 },
        //     &Point { x: 12, y: 3 },
        //     &bitmap
        // ));
    }

    #[test]
    fn test_fill() {
        let mut bitmap = vec![BTreeSet::new(); 5];
        let points = vec![
            Point { x: 1, y: 1 },
            Point { x: 1, y: 3 },
            Point { x: 3, y: 3 },
            Point { x: 3, y: 1 },
        ];
        draw(&points, &mut bitmap);

        assert_eq!(
            vec![
                BTreeSet::new(),
                BTreeSet::from([Horizontal(West, 1..=3, Right)]),
                BTreeSet::from([Vertical(North, 1, Right), Vertical(South, 3, Right)]),
                BTreeSet::from([Horizontal(East, 1..=3, Right)]),
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
            fill_row(&BTreeSet::from([Horizontal(East, 1..=3, Right)]))
        );

        assert_eq!(
            vec![1..=3, 10..=20],
            fill_row(&BTreeSet::from([
                Horizontal(East, 1..=3, Right),
                Horizontal(East, 10..=20, Right)
            ]))
        );

        assert_eq!(
            vec![1..=3],
            fill_row(&BTreeSet::from([
                Vertical(North, 1, Right),
                Vertical(South, 3, Right)
            ]))
        );

        assert_eq!(
            vec![1..=3, 10..=20],
            fill_row(&BTreeSet::from([
                Vertical(North, 1, Right),
                Vertical(South, 3, Right),
                Vertical(North, 10, Left),
                Vertical(South, 20, Right),
            ]))
        );

        assert_eq!(
            vec![1..=20],
            fill_row(&BTreeSet::from([
                Vertical(North, 1, Right),
                Horizontal(East, 10..=20, Left)
            ]))
        );
    }

    fn _example_points() -> Vec<Point> {
        let lines = EXAMPLE_INPUT.split('\n').map(String::from).collect();
        parse_lines(lines)
    }
}
