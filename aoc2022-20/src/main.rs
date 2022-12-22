#![allow(clippy::comparison_chain)]
use anyhow::{Context, Error};
use core::num;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone)]
struct Number {
    id: usize,
    idx: i64,
    value: i64,
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.idx.partial_cmp(&other.idx) {
            Some(core::cmp::Ordering::Equal) => None,
            ord => ord,
        }
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

fn print_numbers(numbers: &[Number]) {
    let mut numbers: Vec<Number> = numbers.to_vec();
    numbers.sort();

    print!("idx  ");
    for num in numbers.iter() {
        print!(" {:>5}", num.idx);
    }
    println!();

    // print!("id   ");
    // for num in numbers.iter() {
    //     print!(" {:>5}", num.id);
    // }
    // println!();

    print!("val  ");
    for num in numbers.iter() {
        print!(" {:>5}", num.value);
    }
    println!();
}

fn do_the_mix(numbers: &mut [Number]) {
    let size = numbers.len();
    for oidx in 0..size {
        let num = numbers[oidx].clone();
        println!("[{oidx}] {num:?}");

        let new_pos = (num.idx + num.value) % (size as i64 - 1);
        let new_pos = if new_pos >= 0 {
            new_pos
        } else {
            size as i64 + new_pos - 1
        };
        dbg!(&new_pos);

        if new_pos == num.idx {
            continue;
        }

        // fix ordering of numbers
        for iidx in 0..size {
            if iidx == oidx {
                // do not update the newly set index
                continue;
            }

            let num2 = &mut numbers[iidx];

            if num.idx < new_pos {
                // moved forwards
                if num2.idx > num.idx && num2.idx <= new_pos {
                    num2.idx -= 1;
                }
            } else if new_pos < num.idx {
                // moved backwards
                if num2.idx >= new_pos && num2.idx < num.idx {
                    num2.idx += 1;
                }
            }
        }

        // apply new idx
        numbers[oidx].idx = new_pos;

        // print_numbers(&numbers);
        // println!();
    }
}

fn solve1(mut numbers: Vec<Number>) -> Result<i64, Error> {
    let size = numbers.len();

    do_the_mix(&mut numbers);

    println!("\n====\n");
    numbers.sort();
    // final idx fix, for easier accessing
    let mut zero_idx: Option<usize> = None;
    for (idx, num) in numbers.iter_mut().enumerate() {
        num.idx = idx as i64;
        if num.value == 0 {
            zero_idx = Some(idx);
        }
    }

    print_numbers(&numbers);

    let zero_idx = zero_idx.ok_or_else(|| Error::msg("Failed to find index of 0"))?;
    let num1000 = &numbers[(zero_idx + 1000) % size];
    let num2000 = &numbers[(zero_idx + 2000) % size];
    let num3000 = &numbers[(zero_idx + 3000) % size];
    println!("1000: {num1000:?}");
    println!("2000: {num2000:?}");
    println!("3000: {num3000:?}");

    let password = num1000.value + num2000.value + num3000.value;
    dbg!(&password);

    Ok(password)
}

fn solve2(mut numbers: Vec<Number>) -> Result<i64, Error> {
    static DECRYPTION_KEY: i64 = 811589153;
    numbers
        .iter_mut()
        .for_each(|num| num.value *= DECRYPTION_KEY);
    let size = numbers.len();

    for _ in 0..10 {
        do_the_mix(&mut numbers);
    }

    println!("\n====\n");
    numbers.sort();
    // final idx fix, for easier accessing
    let mut zero_idx: Option<usize> = None;
    for (idx, num) in numbers.iter_mut().enumerate() {
        num.idx = idx as i64;
        if num.value == 0 {
            zero_idx = Some(idx);
        }
    }

    print_numbers(&numbers);

    let zero_idx = zero_idx.ok_or_else(|| Error::msg("Failed to find index of 0"))?;
    let num1000 = &numbers[(zero_idx + 1000) % size];
    let num2000 = &numbers[(zero_idx + 2000) % size];
    let num3000 = &numbers[(zero_idx + 3000) % size];
    println!("1000: {num1000:?}");
    println!("2000: {num2000:?}");
    println!("3000: {num3000:?}");

    let password = num1000.value + num2000.value + num3000.value;
    dbg!(&password);

    Ok(password)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let mut numbers = BufReader::new(
        File::open(file_name).with_context(|| "Could not open file: '{file_name}'")?,
    )
    .lines()
    .into_iter()
    .filter_map(|line| line.ok())
    .enumerate()
    .map(|(id, line)| -> Result<_, Error> {
        Ok(Number {
            id,
            idx: id as i64,
            value: line.parse()?,
        })
    })
    .collect::<Result<Vec<Number>, _>>()?;

    // print_numbers(&numbers);
    // println!();

    // solve1(numbers)?;
    solve2(numbers)?;

    Ok(())
}
