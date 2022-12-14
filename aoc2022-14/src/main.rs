use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Error, Context};
use itertools::Itertools;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let rock_paths = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| {
            line.split("->")
            .map(|coord| coord.trim().split_once(",").ok_or_else(|| Error::msg(format!("Failed to parse coordinates: '{coord}'"))))
            .map_ok(|(x, y)| -> Result<(i32, i32), Error> { Ok((x.parse::<i32>().context("Failed to parse x")?, y.parse::<i32>().context("Failed to parse y")?))})
            .collect::<Result<Vec<(i32, i32)>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
<
    Ok(())
}
