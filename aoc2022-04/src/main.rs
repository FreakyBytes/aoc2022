use anyhow::Context;
use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn split_range(range: &str) -> Result<(u32, u32), anyhow::Error> {
    let (from, to) = range
        .split_once('-')
        .with_context(|| format!("Wrong format in range: {}", range))?;

    Ok((from.parse()?, to.parse()?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let reader = BufReader::new(File::open(file_name)?);

    let overlapping_pairs = reader
        .lines()
        .into_iter()
        .filter_map(|l| l.ok())
        .map(|l| {
            let Some((group1, group2)) = l.split_once(',') else {
            panic!("Wrong format, no comma: {}", l);
        };
            (split_range(group1).unwrap(), split_range(group2).unwrap())
        })
        // // fully overlapping ranges
        // .filter(|((g1_from, g1_to), (g2_from, g2_to))| {
        //     // group 1 includes 2
        //     g1_from <= g2_from && g1_to >= g2_to
        // // group 2 includes 1
        // || g2_from <= g1_from && g2_to >= g1_to
        // })
        // partly overlapping ranges
        .filter(|((g1_from, g1_to), (g2_from, g2_to))| {
            let g1: HashSet<u32> = HashSet::from_iter((*g1_from)..=(*g1_to));
            let g2: HashSet<u32> = HashSet::from_iter((*g2_from)..=(*g2_to));
            g1.intersection(&g2).count() > 0
        })
        .count();

    println!("Pairs with overlapping ranges: {overlapping_pairs}");

    Ok(())
}
