use itertools::Itertools;
use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

// #[derive(Debug)]
// struct Rucksack {
//     pub comp1: HashSet<char>,
//     pub comp2: HashSet<char>,
// }

// impl Rucksack {
//     pub fn new(line: String) -> Self {
//         let (comp1, comp2) = line.split_at(line.len() / 2);
//         Rucksack {
//             comp1: HashSet::from_iter(comp1.chars()),
//             comp2: HashSet::from_iter(comp2.chars()),
//         }
//     }
// }

fn score_from_char(input: &char) -> u32 {
    if input.is_ascii_lowercase() {
        (*input as u32) - 96
    } else if input.is_ascii_uppercase() {
        (*input as u32) - 64 + 26
    } else {
        panic!("'{}' is not an ascii character!", input);
    }
}

fn div_floor(a: usize, b: u32) -> u32 {
    ((a as f64) / f64::from(b)).floor() as u32
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let reader = BufReader::new(File::open(file_name)?);

    let groups = reader
        .lines()
        .into_iter()
        .map(|line| HashSet::<char>::from_iter(line.unwrap().chars()))
        .enumerate()
        .group_by(|(idx, _)| div_floor(*idx, 3));

    let mut total_score: u32 = 0;
    for (group, rucks) in &groups {
        let badge = rucks
            .map(|(_, ruck)| ruck)
            .reduce(|accum, item| -> HashSet<char> {
                accum.intersection(&item).copied().collect()
            });
        // println!("{}: {:?}", group, badge);
        if let Some(badge) = badge {
            let badge = badge
                .iter()
                .next()
                .ok_or_else(|| anyhow::Error::msg("No badge for you!"))?;

            let score = score_from_char(badge);
            total_score += score;
            println!("{} [{}] -> {}", group, badge, score);
        }
    }

    // for line in reader.lines() {
    //     let line = line?;
    //     let (comp1, comp2) = line.split_at(line.len() / 2);
    //     let comp1: HashSet<char> = HashSet::from_iter(comp1.chars());
    //     let comp2: HashSet<char> = HashSet::from_iter(comp2.chars());

    //     println!("{}:{:?} {}:{:?}", comp1.len(), comp1, comp2.len(), comp2);
    //     if let Some(intersect) = comp1.intersection(&comp2).next() {
    //         let score = score_from_char(intersect);
    //         total_score += score;
    //         println!(" -> {} {}", intersect, score);
    //     }
    // }

    println!("==> {}", total_score);

    Ok(())
}
