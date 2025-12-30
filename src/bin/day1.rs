use std::io::{BufRead, Lines, StdinLock, stdin};

fn main() {
    let lines = stdin().lock().lines();

    let pos = dial(10000050, lines);

    println!("Final result: {pos}");
}

fn dial(pos: i32, lines: Lines<StdinLock>) -> i32 {
    let mut pos = pos;
    let mut tops = 0;

    for line in lines {
        let line = line.unwrap();
        let first_char = line.chars().next().unwrap();
        let dir = match first_char {
            'L' => -1,
            'R' => 1,
            _ => 0,
        };
        let turn = line
            .chars()
            .skip(1)
            .collect::<String>()
            .parse::<i32>()
            .unwrap();

        let new_pos = pos + dir * turn;
        tops += (((pos + dir) / 100) - ((new_pos + dir) / 100)).abs();

        println!("\n{line} position: {} tops: {tops}", new_pos % 100);

        pos = new_pos;
    }

    tops
}
