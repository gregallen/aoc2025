use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};
use std::ops::RangeInclusive;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() < 2 {
        "src/bin/day5.txt"
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

    let result = fresh(text);

    println!("{result}");
}

fn fresh(lines: Vec<String>) -> usize {
    let mut raw_ranges: Vec<RangeInclusive<usize>> = lines
        .iter()
        .take_while(|x| !x.is_empty())
        .map(|r| {
            let (from, to) = r.split_once('-').unwrap();
            println!("from={from}, to={to}");
            from.parse::<usize>().unwrap()..=to.parse::<usize>().unwrap()
        })
        .collect();
    raw_ranges.sort_by_key(|r| *r.start());

    let mut ranges: Vec<RangeInclusive<usize>> = Vec::with_capacity(raw_ranges.len());
    ranges.push(raw_ranges.first().unwrap().clone());

    for raw_range in raw_ranges.iter().skip(1) {
        let last = ranges.last().unwrap().clone();
        if raw_range.start() > last.end() {
            ranges.push(raw_range.clone());
        } else {
            ranges.pop();
            ranges.push(*last.start()..=*last.end().max(raw_range.end()));
        }
    }

    ranges.iter().map(|r| r.end() - r.start() + 1).sum()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn example() {
        let lines = r"3-5
10-14
16-20
12-18

1
5
8
11
17
32"
        .split('\n')
        .map(String::from)
        .collect();
        let result = fresh(lines);
        assert_eq!(14, result);
    }
}
