use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Error;
use itertools::{iproduct, Itertools};
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::*;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Vec2 {
    x: i64,
    y: i64,
}

impl Vec2 {
    fn manhatten_distance(&self, other: &Self) -> i64 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as i64
    }
}

#[derive(Debug, Clone)]
struct Sensor {
    position: Vec2,
    closest_beacon: Vec2,
}

impl Sensor {
    fn cleared_fields(&self) -> impl Iterator<Item = Vec2> {
        let sensor = self.position.clone();
        let beacon = self.closest_beacon.clone();
        let distance = sensor.manhatten_distance(&self.closest_beacon);
        iproduct!(
            (self.position.x - distance)..(self.position.x + distance),
            (self.position.y - distance)..(self.position.y + distance)
        )
        .filter_map(move |(x, y)| {
            let pos = Vec2 { x, y };
            if pos.manhatten_distance(&sensor) <= distance
            // && pos.x != beacon.x && pos.y != beacon.y
            {
                Some(pos)
            } else {
                None
            }
        })
    }

    fn cleared_fields_for_row(&self, y_of_interest: i64) -> impl Iterator<Item = Vec2> {
        let sensor = self.position.clone();
        // let beacon = self.closest_beacon.clone();
        let distance = sensor.manhatten_distance(&self.closest_beacon);
        ((self.position.x - distance)..(self.position.x + distance)).filter_map(move |x| {
            let pos = Vec2 {
                x,
                y: y_of_interest,
            };
            if pos.manhatten_distance(&sensor) <= distance
            // && pos.x != beacon.x && pos.y != beacon.y
            {
                Some(pos)
            } else {
                None
            }
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");

    let sensor_regex = Regex::new(
        r#"Sensor.*x=(?P<sx>[0-9\-]+),\s*y=(?P<sy>[0-9\-]+):.*x=(?P<bx>[0-9\-]+),\s*y=(?P<by>[0-9\-]+)"#,
    )?;
    let sensors = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| {
            let cap = sensor_regex
                .captures(&line)
                .ok_or_else(|| Error::msg("Did not match"))?;
            if let (Some(sx), Some(sy), Some(bx), Some(by)) = (
                cap.name("sx"),
                cap.name("sy"),
                cap.name("bx"),
                cap.name("by"),
            ) {
                Ok(Sensor {
                    position: Vec2 {
                        x: sx.as_str().parse()?,
                        y: sy.as_str().parse()?,
                    },
                    closest_beacon: Vec2 {
                        x: bx.as_str().parse()?,
                        y: by.as_str().parse()?,
                    },
                })
            } else {
                Err(Error::msg("Line does not follow the format!"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // dbg!(&sensors);

    let y_of_interest = 2000000;
    dbg!(y_of_interest);
    let beacons = sensors
        .iter()
        .map(|s| s.closest_beacon.clone())
        .collect::<BTreeSet<_>>();
    dbg!(&beacons);

    // let mut fields_without_beacons = BTreeSet::<Vec2>::new();
    let fields_without_beacon = sensors
        .par_iter()
        .flat_map(|sensor| {
            sensor
                .cleared_fields_for_row(y_of_interest)
                .collect::<BTreeSet<_>>()
        })
        .filter(|pos| !beacons.contains(pos))
        .collect::<BTreeSet<_>>();

    // let fields_on_column = fields_without_beacon
    //     .par_iter()
    //     .filter(|Vec2 { x: _, y }| *y == 10)
    //     .count();

    println!(
        "size = {}",
        fields_without_beacon.len(),
        // fields_on_column
    );
    Ok(())
}
