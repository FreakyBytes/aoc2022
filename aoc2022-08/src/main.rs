use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;

// struct Grid {
//     grid: Vec<Vec<u32>>
// }

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let lines = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok());

    let grid: Vec<Vec<u32>> = lines
        .map(|line| {
            line.chars()
                .filter_map(|c| c.to_digit(10))
                .collect::<Vec<u32>>()
        })
        .collect();

    let mut visible_trees: u32 = 0;
    for (y, row) in grid.iter().enumerate() {
        for (x, tree) in row.iter().enumerate() {
            // check if tree is on the edge
            if x == 0 || y == 0 || x == row.len() - 1 || y == grid.len() - 1 {
                visible_trees += 1;
                continue;
            }

            // check if any other tree is higher or has the same hight as the current tree
            let mut visible_from_left = true;
            let mut visible_from_right = true;
            for (other_x, other_tree) in row.iter().enumerate() {
                if other_x < x && other_tree >= tree {
                    visible_from_left = false;
                }
                if other_x > x && other_tree >= tree {
                    visible_from_right = false;
                }
            }

            let mut visible_from_top = true;
            let mut visible_from_bottom = true;
            for (other_y, other_tree) in
                grid.iter().enumerate().filter_map(|(other_y, other_row)| {
                    other_row.get(x).map(|other_tree| (other_y, other_tree))
                })
            {
                if other_y < y && other_tree >= tree {
                    visible_from_top = false;
                }
                if other_y > y && other_tree >= tree {
                    visible_from_bottom = false;
                }
            }

            if visible_from_left || visible_from_right || visible_from_top || visible_from_bottom {
                visible_trees += 1;
            }
        }
    }

    println!("visible_trees: {visible_trees}");
    Ok(())
}
