// use std::io::{BufRead, Lines, StdinLock, stdin};

fn main() {
    // let lines = stdin().lock().lines();

    // let pos = invalids(10000050, lines);

    // println!("Final result: {pos}");

    let result = invalids(
        "749639-858415,65630137-65704528,10662-29791,1-17,9897536-10087630,1239-2285,1380136-1595466,8238934-8372812,211440-256482,623-1205,102561-122442,91871983-91968838,62364163-62554867,3737324037-3737408513,9494926669-9494965937,9939271919-9939349036,83764103-83929201,24784655-24849904,166-605,991665-1015125,262373-399735,557161-618450,937905586-937994967,71647091-71771804,8882706-9059390,2546-10476,4955694516-4955781763,47437-99032,645402-707561,27-86,97-157,894084-989884,421072-462151",
    );

    println!("{result}");
}

fn invalids(input: &str) -> u64 {
    let mut sum: u64 = 0;

    for pair in input.trim().split(',') {
        let mut range = pair.split('-').map(|x| x.parse::<u64>().unwrap());
        let start = range.next().unwrap();
        let end = range.next().unwrap();

        'candidate: for candidate in start..=end {
            let digits = 1 + (candidate as f64).log10().floor() as u32;
            let half = digits >> 1;
            let mut divisor: u128 = 1;
            for repeated_digits in 1..=half {
                divisor *= 10;
                if digits.is_multiple_of(repeated_digits) {
                    let multiplier = divisor.pow(digits / repeated_digits);
                    let lower = candidate as u128 % divisor;
                    let repeated = (multiplier - 1) / (divisor - 1);
                    if candidate as u128 == lower * repeated {
                        sum += candidate;
                        continue 'candidate;
                    }
                }
            }
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use crate::invalids;

    #[test]
    fn example() {
        let result = invalids(
            "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124",
        );
        assert_eq!(1227775554, result);
    }
}
