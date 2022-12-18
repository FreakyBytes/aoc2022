use anyhow::{Context, Error};
use glam::{IVec3, UVec3};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let cubes = BufReader::new(
        File::open(file_name).with_context(|| "Could not open file: '{file_name}'")?,
    )
    .lines()
    .into_iter()
    .filter_map(|line| line.ok())
    .map(|line| {
        line.splitn(3, ',')
            .map(|val| {
                val.parse::<i32>()
                    .with_context(|| format!("failed to parse '{val}' as number in: '{line}'"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|v| IVec3::from_slice(&v))
    })
    .collect::<Result<HashSet<_>, Error>>()?;
    // .collect::<Result<Vec<_>, Error>>()?;

    let surface_area: usize = cubes
        .iter()
        .map(|&p| {
            let occupied_sizes: usize = [
                p - IVec3::X,
                p + IVec3::X,
                p - IVec3::Y,
                p + IVec3::Y,
                p - IVec3::Z,
                p + IVec3::Z,
            ]
            .iter()
            .filter(|lookup| cubes.contains(&lookup))
            .count();
            6 - occupied_sizes
        })
        .sum();

    println!("Surface area is {surface_area}");
    Ok(())
}
