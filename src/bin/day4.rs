use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};
use std::{env, mem};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    // Try to open the file, fall back to stdin if no filename is provided

    let file = if filename == "-" {
        Box::new(stdin().lock()) as Box<dyn BufRead>
    } else {
        Box::new(BufReader::new(
            File::open(filename).expect("Failed to open file"),
        ))
    };
    let lines = file.lines();
    let text = lines.map(Result::unwrap).collect::<Vec<_>>();

    let result = paper(text);

    println!("{result}");
}

fn paper(mut lines: Vec<String>) -> u64 {
    let mut removed = 0;

    let mut current_lines = &mut lines;
    let mut next_lines_vec: Vec<String> = (*current_lines).clone();

    let mut next_lines = &mut next_lines_vec;

    loop {
        let last_removed = removed;
        for (i, line) in current_lines.iter().enumerate() {
            let next_line = &mut next_lines[i];
            next_line.clear();
            next_line.push_str(line);
            for (j, c) in line.chars().enumerate() {
                let mut neighbours = 0;
                if c == '@' {
                    if i > 0 {
                        neighbours += check3(&current_lines[i - 1], j)
                    }
                    neighbours += check3(&current_lines[i], j);
                    if i + 1 < current_lines.len() {
                        neighbours += check3(&current_lines[i + 1], j)
                    }
                    if neighbours <= 4 {
                        removed += 1;
                        next_line.replace_range(j..=j, ".");
                    }
                }
            }
        }

        if removed == last_removed {
            break;
        }

        mem::swap(&mut next_lines, &mut current_lines);
    }

    removed
}

#[inline]
fn check3(line: &str, pos: usize) -> u64 {
    let left = max(1, pos) - 1;
    let right = min(line.len() - 1, pos + 1);
    line[left..=right].chars().filter(|&c| c == '@').count() as u64
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn example() {
        let lines = r"..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@."
            .split('\n')
            .map(String::from)
            .collect();
        let result = paper(lines);
        assert_eq!(43, result);
    }

    #[test]
    fn simple() {
        assert_eq!(2, check3("@@@", 0));
        assert_eq!(3, check3("@@@", 1));
        assert_eq!(2, check3("@@@", 2));
    }
}
