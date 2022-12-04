use anyhow::Context;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use itertools::Itertools;

fn split_range(range: &str) -> Result<(u32, u32), anyhow::Error> {
    let (from, to) = range
        .split_once('-')
        .with_context(|| format!("Wrong format in range: {}", range))?;

    Ok((from.parse()?, to.parse()?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let reader = BufReader::new(File::open(file_name)?);

    let pairs_with_covered_range = reader
        .lines()
        .into_iter()
        .filter_map(|l| l.ok())
        .map(|l| {
            let Some((group1, group2)) = l.split_once(',') else {
            panic!("Wrong format, no comma: {}", l);
        };
            (split_range(group1).unwrap(), split_range(group2).unwrap())
        })
        .filter(|((g1_from, g1_to), (g2_from, g2_to))| {
            // group 1 includes 2
            g1_from <= g2_from && g1_to >= g2_to
        // group 2 includes 1
        || g2_from <= g1_from && g2_to >= g1_to
        })
        .count();

    println!("Pairs with fully overlapping ranges: {pairs_with_covered_range}");

    Ok(())
}
