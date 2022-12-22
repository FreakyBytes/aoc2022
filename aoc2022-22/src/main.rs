use anyhow::{Context, Error, Result};
use glam::{IVec2, UVec2};
use itertools::Itertools;
use parser::{parse_input, Grid, GridCell, WalkRule};

use crate::parser::Direction;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;

fn visualize_grid(grid: &Grid) {
    println!();
    for row in grid.iter() {
        for cell in row.iter() {
            match cell {
                parser::GridCell::Void => print!(" "),
                parser::GridCell::Rock => print!("#"),
                parser::GridCell::Empty(None) => print!("."),
                parser::GridCell::Empty(Some(Direction::North)) => print!("^"),
                parser::GridCell::Empty(Some(Direction::South)) => print!("v"),
                parser::GridCell::Empty(Some(Direction::East)) => print!(">"),
                parser::GridCell::Empty(Some(Direction::West)) => print!("<"),
            }
        }
        println!();
    }
}

fn get_start_position(grid: &Grid) -> Result<UVec2> {
    grid.get(0)
        .map(|top_row| {
            top_row
                .iter()
                .find_position(|c| matches!(**c, GridCell::Empty(_)))
                .ok_or(Error::msg("Did not find any Empty cell in top row"))
        })
        .map_or_else(
            || Err(Error::msg("Grid has no top row!")),
            |r| match r {
                Ok(v) => Ok(UVec2::new(v.0 as u32, 0)),
                Err(err) => Err(err),
            },
        )
}

fn get_next_grid_cell(grid: &Grid, position: UVec2, direction: &Direction) -> (UVec2, GridCell) {
    // these are only for wrapping around and to prevent overflow
    let last_column = (grid[position.y as usize].len() - 1) as u32;
    let (last_row, _) = grid
        .iter()
        .rev()
        .find_position(|row| row.get(position.x as usize).is_some())
        .unwrap();
    let last_row = (grid.len() - last_row - 1) as u32;
    // let last_row = (grid.len() - 1) as u32;

    let new_pos = match (position, direction) {
        // special, edge of grid, cases
        (UVec2 { x: 0, y }, Direction::West) => UVec2::new(last_column, y),
        (UVec2 { x, y }, Direction::East) if x == last_column => UVec2::new(0, y),
        (UVec2 { x, y: 0 }, Direction::North) => UVec2::new(x, last_row),
        (UVec2 { x, y }, Direction::South) if y == last_row => UVec2::new(x, 0),
        // normal cases
        (pos, Direction::North) => pos - UVec2::Y,
        (pos, Direction::South) => pos + UVec2::Y,
        (pos, Direction::West) => pos - UVec2::X,
        (pos, Direction::East) => pos + UVec2::X,
    };

    match &grid[new_pos.y as usize][new_pos.x as usize] {
        GridCell::Void => {
            // we must go further, because there is this void on the map
            get_next_grid_cell(grid, new_pos, direction)
        }
        c @ (GridCell::Empty(_) | GridCell::Rock) => {
            // reached something valuable
            (new_pos, c.clone())
        }
    }
}

fn solve1(mut grid: Grid, walk_rules: &Vec<WalkRule>) -> Result<u32> {
    visualize_grid(&grid);

    // let mut grid = grid.clone();
    let mut walk_direction = Direction::East;
    let mut position: UVec2 = get_start_position(&grid)?;
    grid[position.y as usize][position.x as usize] = GridCell::Empty(Some(walk_direction.clone()));
    for rule in walk_rules.iter() {
        match rule {
            parser::WalkRule::TurnLeft => {
                walk_direction = walk_direction.turn_left();
                grid[position.y as usize][position.x as usize] =
                    GridCell::Empty(Some(walk_direction.clone()));
                continue;
            }
            parser::WalkRule::TurnRight => {
                walk_direction = walk_direction.turn_right();
                grid[position.y as usize][position.x as usize] =
                    GridCell::Empty(Some(walk_direction.clone()));
                continue;
            }
            parser::WalkRule::Step(length) => {
                for _ in 0..*length {
                    let (next_pos, cell) = get_next_grid_cell(&grid, position, &walk_direction);
                    match cell {
                        GridCell::Empty(_) => {
                            // actually do the step
                            position = next_pos;
                            grid[position.y as usize][position.x as usize] =
                                GridCell::Empty(Some(walk_direction.clone()));
                        }
                        GridCell::Rock => {
                            // do not move position, since we can't
                            break;
                        }
                        GridCell::Void => {} // shouldn't happen anyway
                    }
                }
            }
        }
        println!("\n----\n");
        visualize_grid(&grid);
    }

    println!();
    dbg!(position, &walk_direction);
    let password = (1000 * (position.y + 1)) + (4 * (position.x + 1)) + (walk_direction as u32);
    dbg!(&password);

    Ok(password)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let (grid, walk_rules) = parse_input(&file_name)?;
    dbg!(&walk_rules);

    solve1(grid.clone(), &walk_rules)?;

    // 02
    // find cutting points
    // use them to split into faces
    // store immediate adjacency of faces
    // profit

    Ok(())
}
