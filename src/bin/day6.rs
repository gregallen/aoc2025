use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() < 2 {
        "src/bin/day6.txt"
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

    let result = homework2(text);

    println!("{result}");
}

fn homework(lines: Vec<String>) -> u64 {
    let mut split_lines: Vec<_> = lines
        .iter()
        .map(|l| {
            l.split(' ')
                .filter(|l| !l.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .collect();

    let ops: Vec<_> = split_lines.pop().unwrap();

    let mut result = 0;

    for (i, op) in ops.iter().enumerate() {
        let column = split_lines
            .iter()
            .map(|v| v.get(i).unwrap().parse::<u64>().unwrap());
        let partial = if op == "+" {
            column.sum::<u64>()
        } else {
            column.product()
        };
        // println!("{i}\t{op}\t{partial}");
        result += partial;
    }
    result
}

fn homework2(mut lines: Vec<String>) -> u64 {
    let ops = lines.pop().unwrap();

    let mut result = 0;
    let mut partial = 0u64;
    let mut op = '?';

    for col in 0..ops.len() {
        let maybe_op = ops.chars().nth(col).unwrap();
        if maybe_op != ' ' {
            println!("updating {result} += {partial}");
            result += partial;
            op = maybe_op;
            partial = if op == '+' { 0 } else { 1 };
        }
        let mut operand = 0u64;
        for line in &lines {
            if let Ok(digit) = line
                .chars()
                .nth(col)
                .unwrap_or(' ')
                .to_string()
                .parse::<u64>()
            {
                operand = operand * 10 + digit;
            }
        }
        if operand != 0 {
            if op == '+' {
                partial += operand;
            } else {
                partial *= operand;
            }
        }
        println!("{op}:  {operand}\t{partial}\t{result}");
    }

    // for (i, op) in ops.iter().enumerate() {
    //
    // }
    result + partial
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn example() {
        let lines = r"123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +  "
            .split('\n')
            .map(String::from)
            .collect();
        let result = homework(lines);
        assert_eq!(4277556, result);
    }

    #[test]
    fn example2() {
        let lines = r"123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +  "
            .split('\n')
            .map(String::from)
            .collect();
        let result = homework2(lines);
        assert_eq!(3263827, result);
    }
}
