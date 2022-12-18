use anyhow::{Context, Error};
use glam::IVec3;
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

fn find_bbox(cubes: &HashSet<IVec3>) -> (IVec3, IVec3) {
    cubes.iter().fold(
        (
            IVec3::new(i32::MAX, i32::MAX, i32::MAX),
            IVec3::new(i32::MIN, i32::MIN, i32::MIN),
        ),
        |(min, max), p| (min.min(*p), max.max(*p)),
    )
}

fn enlarge_bbox((min, max): (IVec3, IVec3), v: i32) -> (IVec3, IVec3) {
    (
        IVec3::new(min.x - v, min.y - v, min.z - v),
        IVec3::new(max.x + v, max.y + v, max.z + v),
    )
}

fn flood_fill(cubes: &HashSet<IVec3>, bbox: (IVec3, IVec3)) -> HashSet<IVec3> {
    let mut water = HashSet::<IVec3>::new();
    water.insert(bbox.0);
    water.insert(bbox.1);

    loop {
        let to_add = water
            .iter()
            .flat_map(|&p| {
                [
                    p - IVec3::X,
                    p + IVec3::X,
                    p - IVec3::Y,
                    p + IVec3::Y,
                    p - IVec3::Z,
                    p + IVec3::Z,
                ]
                .into_iter()
                .filter(|&IVec3 { x, y, z }| {
                    x >= bbox.0.x
                        && x <= bbox.1.x
                        && y >= bbox.0.y
                        && y <= bbox.1.y
                        && z >= bbox.0.z
                        && z <= bbox.1.z
                })
                .filter(|l| !cubes.contains(l))
                .filter(|l| !water.contains(l))
                .collect::<HashSet<IVec3>>()
            })
            .collect::<HashSet<_>>();

        if to_add.is_empty() {
            break;
        }
        water = water.union(&to_add).copied().collect::<HashSet<IVec3>>();
    }

    water
}

fn sum_wet_surface_area(cubes: &HashSet<IVec3>, water: &HashSet<IVec3>) -> usize {
    cubes
        .iter()
        .map(|&p| {
            [
                p - IVec3::X,
                p + IVec3::X,
                p - IVec3::Y,
                p + IVec3::Y,
                p - IVec3::Z,
                p + IVec3::Z,
            ]
            .into_iter()
            .filter(|lookup| !cubes.contains(lookup))
            .filter(|lookup| water.contains(lookup))
            .count()
        })
        .sum()
}

fn sum_surface_area(cubes: &HashSet<IVec3>) -> usize {
    cubes
        .iter()
        .map(|&p| {
            let unoccupied_sides: usize = [
                p - IVec3::X,
                p + IVec3::X,
                p - IVec3::Y,
                p + IVec3::Y,
                p - IVec3::Z,
                p + IVec3::Z,
            ]
            .iter()
            .filter(|lookup| !cubes.contains(lookup))
            .count();
            unoccupied_sides
        })
        .sum()
}

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

    let surface_area = sum_surface_area(&cubes);
    println!("Total surface area is {surface_area}");
    let bbox = find_bbox(&cubes);
    println!("Bounding box {bbox:?}");
    let bbox = enlarge_bbox(bbox, 1);
    println!("Enlarged box {bbox:?}");

    let water = flood_fill(&cubes, bbox);
    println!("Filled bbox with {} blocks of water", water.len());

    let wet_surface_area = sum_wet_surface_area(&cubes, &water);
    println!("Wet surface area is {wet_surface_area}");

    Ok(())
}
