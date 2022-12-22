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

fn print_numbers(numbers: &Vec<Number>) {
    let mut numbers = numbers.clone();
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
    let size = numbers.len();

    print_numbers(&numbers);
    println!();

    // let mut idx: usize = 0;
    // let mut already_mixed: BTreeSet<usize> = BTreeSet::new();
    // while idx < numbers.len() && already_mixed.len() < numbers.len() {
    //     let num = &numbers[idx];
    //     println!("[{idx}] {num:?}");
    //     if already_mixed.contains(&num.id) || num.value == 0 {
    //         idx += 1;
    //         continue;
    //     }

    //     let new_pos = (idx as i64 + num.value) % (numbers.len() as i64);
    //     let new_pos = if new_pos >= 0 {
    //         new_pos as usize
    //     } else {
    //         (numbers.len() as i64 + new_pos - 1) as usize
    //     };
    //     dbg!(&new_pos);

    //     let num = numbers.remove(idx);
    //     already_mixed.insert(num.id);
    //     numbers.insert(new_pos, num);

    //     if new_pos > idx {
    //         // we moved the number forward, so on the current position a now new number appeared
    //     } else if new_pos <= idx {
    //         // we moved the number behind, to keep up, we need to move forward
    //         // - or -
    //         // the index remained the same, we need to move forward
    //         idx += 1
    //     }

    //     print_numbers(&numbers);
    //     println!();
    // }

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
                if num.idx < num2.idx && num2.idx <= new_pos {
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
    dbg!(password);

    Ok(())
}
