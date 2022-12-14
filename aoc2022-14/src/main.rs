use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Error};
use itertools::{iproduct, Itertools};
use ndarray::Array2;

const START: (i32, i32) = (500, 0);

#[derive(Debug, PartialEq, Eq)]
enum SandState {
    Moving,
    Rest,
}

#[derive(Debug, PartialEq, Eq)]
enum GridState {
    Air,
    Rock,
    Sand(SandState),
    Spawn,
}
impl Default for GridState {
    fn default() -> Self {
        Self::Air
    }
}

type Grid = Array2<GridState>;

fn render_grid(grid: &Grid) {
    for col in grid.columns() {
        for cell in col {
            match cell {
                GridState::Air => print!("."),
                GridState::Rock => print!("#"),
                GridState::Sand(SandState::Moving) => print!("~"),
                GridState::Sand(SandState::Rest) => print!("o"),
                GridState::Spawn => print!("+"),
            }
        }
        println!();
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let rock_paths = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| {
            line.split("->")
                .map(|coord| {
                    coord
                        .trim()
                        .split_once(',')
                        .ok_or_else(|| {
                            Error::msg(format!("Failed to parse coordinates: '{coord}'"))
                        })
                        .map(|(x, y)| {
                            Ok((
                                x.parse::<i32>().context("Failed to parse x")?,
                                y.parse::<i32>().context("Failed to parse y")?,
                            ))
                        })?
                })
                .collect::<Result<Vec<(i32, i32)>, Error>>()
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let (x_min, x_max, y_min, y_max) = rock_paths
        .iter()
        .map(|path| {
            path.iter().fold(
                (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                |(x_min, x_max, y_min, y_max), (x, y)| {
                    (x_min.min(*x), x_max.max(*x), y_min.min(*y), y_max.max(*y))
                },
            )
        })
        .fold(
            (
                250,
                750,
                // START.0 as i32,
                // START.0 as i32,
                START.1 as i32,
                START.1 as i32,
            ),
            |(x0_min, x0_max, y0_min, y0_max), (x1_min, x1_max, y1_min, y1_max)| {
                (
                    x0_min.min(x1_min),
                    x0_max.max(x1_max),
                    y0_min.min(y1_min),
                    y0_max.max(y1_max),
                )
            },
        );
    dbg!((x_min, x_max, y_min, y_max));

    let spawn = ((START.0 - x_min) as usize, (START.1 - y_min) as usize);
    let grid_size = ((x_max - x_min) as usize + 1, (y_max - y_min + 3) as usize);
    let mut grid = Grid::default(grid_size);

    for path in rock_paths.iter() {
        for ((x0, y0), (x1, y1)) in path.iter().tuple_windows::<(_, _)>() {
            if x0 == x1 {
                for y in i32::min(*y0, *y1)..=i32::max(*y0, *y1) {
                    grid[[(*x0 - x_min) as usize, (y - y_min) as usize]] = GridState::Rock;
                }
            } else if y0 == y1 {
                for x in i32::min(*x0, *x1)..=i32::max(*x0, *x1) {
                    grid[[(x - x_min) as usize, (y0 - y_min) as usize]] = GridState::Rock;
                }
            } else {
                panic!("No straight line!");
            }
        }

        // grid[[(x - x_min) as usize, (y - y_min) as usize]] = GridState::Rock;
    }
    // grid[[spawn.0, spawn.1]] = GridState::Spawn;
    for x in 0..grid_size.0 {
        grid[[x, grid_size.1 - 1]] = GridState::Rock;
    }

    render_grid(&grid);

    let mut units: u32 = 0;
    let mut units_at_rest: u32 = 0;
    loop {
        let units_before = units;
        if grid[[spawn.0, spawn.1]] != GridState::Air {
            println!("Units reached spawn!");
            break;
        } else {
            units += 1;
            grid[[spawn.0, spawn.1]] = GridState::Sand(SandState::Moving);
        }

        for (y, x) in iproduct!((0..grid_size.1).rev(), 0..grid_size.0) {
            // for y in (0..grid_size.1).rev() {
            //     for x in 0..grid_size.0 {
            match grid[[x, y]] {
                // sand falls out of area
                GridState::Sand(SandState::Moving) if y >= grid_size.1 - 1 => {
                    grid[[x, y]] = GridState::Air;
                    units -= 1;
                    panic!("Sand fell off");
                }
                // falling straight down
                GridState::Sand(SandState::Moving) if grid[[x, y + 1]] == GridState::Air => {
                    grid[[x, y]] = GridState::Air;
                    grid[[x, y + 1]] = GridState::Sand(SandState::Moving);
                }
                // falling left out of the area
                GridState::Sand(SandState::Moving)
                    if grid[[x, y + 1]] != GridState::Air && x == 0 =>
                {
                    grid[[x, y]] = GridState::Air;
                    units -= 1;
                    panic!("Sand fell off");
                }
                // falling to the left
                GridState::Sand(SandState::Moving)
                    if grid[[x, y + 1]] != GridState::Air
                        && grid[[x - 1, y + 1]] == GridState::Air =>
                {
                    grid[[x, y]] = GridState::Air;
                    grid[[x - 1, y + 1]] = GridState::Sand(SandState::Moving);
                }
                // falling right out of the area
                GridState::Sand(SandState::Moving)
                    if grid[[x, y + 1]] != GridState::Air && x >= grid_size.0 - 1 =>
                {
                    grid[[x, y]] = GridState::Air;
                    units -= 1;
                    panic!("Sand fell off");
                }
                // falling to the right
                GridState::Sand(SandState::Moving)
                    if grid[[x, y + 1]] != GridState::Air
                        && grid[[x + 1, y + 1]] == GridState::Air =>
                {
                    grid[[x, y]] = GridState::Air;
                    grid[[x + 1, y + 1]] = GridState::Sand(SandState::Moving);
                }
                // finally this sand is not moving anymore
                GridState::Sand(SandState::Moving) => {
                    grid[[x, y]] = GridState::Sand(SandState::Rest);
                    units_at_rest += 1;
                }
                _ => {}
            }
            // }
        }
        println!("units = {units}  |  units at rest = {units_at_rest}");
        // render_grid(&grid);

        if units == units_before {
            println!("Units stopped increasing!");
            break;
        }
    }

    dbg!(units, units_at_rest);
    render_grid(&grid);

    Ok(())
}
