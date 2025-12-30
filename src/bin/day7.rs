use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() < 2 {
        "src/bin/day7.txt"
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

    let result = teleport2(text);

    println!("{result}");
}

fn teleport(lines: Vec<String>) -> u64 {
    let first = &lines[0];
    let beam = first.find('S').unwrap();
    println!("Beam: {}", beam);

    let mut splits = 0;
    let mut beams = bit_vec::BitVec::from_elem(first.len(), false);
    beams.set(beam, true);

    for line in lines.iter().skip(1) {
        line.chars()
            .enumerate()
            .filter(|(_i, c)| *c == '^')
            .for_each(|(i, _)| {
                beams.set(i - 1, true);
                beams.set(i, false);
                beams.set(i + 1, true);
                splits += 1;
            });
        println!("Bitvec: {}", beams);
    }

    splits
}

fn teleport2(lines: Vec<String>) -> u64 {
    let first = &lines[0];
    let beam = first.find('S').unwrap();
    println!("Beam: {}", beam);

    let mut beams = vec![0u64; first.len()];
    beams[beam] = 1;

    for line in lines.iter().skip(1) {
        line.chars()
            .enumerate()
            .filter(|(_i, c)| *c == '^')
            .for_each(|(i, _c)| {
                beams[i - 1] += beams[i];
                beams[i + 1] += beams[i];
                beams[i] = 0;
            });
        println!("{:?}", beams);
    }

    beams.iter().sum()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn example() {
        let lines = r".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
..............."
            .split('\n')
            .map(String::from)
            .collect();
        let result = teleport(lines);
        assert_eq!(21, result);
    }

    #[test]
    fn example2() {
        let lines = r".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
..............."
            .split('\n')
            .map(String::from)
            .collect();
        let result = teleport2(lines);
        assert_eq!(40, result);
    }
}
