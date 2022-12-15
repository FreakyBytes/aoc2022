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
    distance: i64,
}

impl Sensor {
    fn new(position: Vec2, closest_beacon: Vec2) -> Self {
        Self {
            distance: position.manhatten_distance(&closest_beacon),
            position,
            closest_beacon,
        }
    }
    // fn cleared_fields(&self) -> impl Iterator<Item = Vec2> {
    //     let sensor = self.position.clone();
    //     let beacon = self.closest_beacon.clone();
    //     let distance = self.distance;
    //     iproduct!(
    //         (self.position.x - distance)..(self.position.x + distance),
    //         (self.position.y - distance)..(self.position.y + distance)
    //     )
    //     .filter_map(move |(x, y)| {
    //         let pos = Vec2 { x, y };
    //         if pos.manhatten_distance(&sensor) <= distance
    //         // && pos.x != beacon.x && pos.y != beacon.y
    //         {
    //             Some(pos)
    //         } else {
    //             None
    //         }
    //     })
    // }

    fn cleared_fields_for_row(&self, y_of_interest: i64) -> impl Iterator<Item = Vec2> {
        let sensor = self.position.clone();
        // let beacon = self.closest_beacon.clone();
        let distance = self.distance;
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

    fn get_border_fields(&self) -> impl Iterator<Item = Vec2> {
        // fn get_border_fields(&self) -> impl ParallelIterator<Item = Vec2> {
        let sensor = self.position.clone();
        let distance = self.distance + 1;
        iproduct!(
            (self.position.x - distance).max(0)..(self.position.x + distance).min(4000000),
            (self.position.y - distance).max(0)..(self.position.y + distance).min(4000000)
        )
        // .par_bridge()
        .filter_map(move |(x, y)| {
            let pos = Vec2 { x, y };
            if pos.manhatten_distance(&sensor) == distance {
                Some(pos)
            } else {
                None
            }
        })
    }

    fn is_an_undetected_beacon_out_of_reach(&self, pos: &Vec2) -> bool {
        // (pos.x != self.closest_beacon.x && pos.y != self.closest_beacon.y)
        //     &&
        self.position.manhatten_distance(pos) > self.distance
    }

    fn radius_iter(&self) -> RadiusIterator {
        RadiusIterator {
            pos: self.position.clone(),
            distance: self.distance + 1,
            y_offset: 0,
            direction: 0,
        }
    }
}

struct RadiusIterator {
    pos: Vec2,
    distance: i64,

    y_offset: i64,
    direction: u8,
}

impl Iterator for RadiusIterator {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        match self.direction % 4 {
            0 => {
                self.direction += 1;
                Some(Vec2 {
                    x: self.pos.x - (self.distance - self.y_offset),
                    y: self.pos.y - self.y_offset,
                })
            }
            1 => {
                self.direction += 1;
                Some(Vec2 {
                    x: self.pos.x + (self.distance - self.y_offset),
                    y: self.pos.y - self.y_offset,
                })
            }
            2 => {
                self.direction += 1;
                Some(Vec2 {
                    x: self.pos.x - (self.distance - self.y_offset),
                    y: self.pos.y + self.y_offset,
                })
            }
            3 if self.y_offset > self.distance => None,
            3 => {
                self.direction = 0;
                let r = Some(Vec2 {
                    x: self.pos.x + (self.distance - self.y_offset),
                    y: self.pos.y + self.y_offset,
                });
                self.y_offset += 1;
                r
            }
            _ => panic!("fuck"),
        }
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
                Ok(Sensor::new(
                    Vec2 {
                        x: sx.as_str().parse()?,
                        y: sy.as_str().parse()?,
                    },
                    Vec2 {
                        x: bx.as_str().parse()?,
                        y: by.as_str().parse()?,
                    },
                ))
            } else {
                Err(Error::msg("Line does not follow the format!"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // dbg!(&sensors);

    // part 1
    let y_of_interest = 2000000;
    dbg!(y_of_interest);
    let beacons = sensors
        .iter()
        .map(|s| s.closest_beacon.clone())
        .collect::<BTreeSet<_>>();
    // dbg!(&beacons);

    // // let mut fields_without_beacons = BTreeSet::<Vec2>::new();
    let fields_without_beacon = sensors
        .par_iter()
        .flat_map(|sensor| {
            sensor
                .cleared_fields_for_row(y_of_interest)
                .collect::<BTreeSet<_>>()
        })
        .filter(|pos| !beacons.contains(pos))
        .collect::<BTreeSet<_>>();

    println!("size = {}", fields_without_beacon.len(),);

    // part 2

    // let coord_max = 20;
    let coord_max = 4000000;

    sensors
        .par_iter()
        .flat_map(|sense| sense.radius_iter().collect::<Vec<_>>())
        .filter(|Vec2 { x, y }| *x >= 0 && *x <= coord_max && *y >= 0 && *y <= coord_max)
        .filter(|pos| {
            !sensors
                .iter()
                .any(|s| !s.is_an_undetected_beacon_out_of_reach(&pos))
        })
        .map(|Vec2 { x, y }| (x, y, (x * 4000000) + y))
        .for_each(|r| {
            dbg!(&r);
        });

    Ok(())
}
