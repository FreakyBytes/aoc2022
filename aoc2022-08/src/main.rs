use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

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
    let mut highest_scenic_score: u32 = 0;
    let mut scenic_score_position: (usize, usize) = (0, 0);
    for (y, row) in grid.iter().enumerate() {
        for (x, tree) in row.iter().enumerate() {
            // check if tree is on the edge
            // if x == 0 || y == 0 || x == row.len() - 1 || y == grid.len() - 1 {
            //     visible_trees += 1;
            //     continue;
            // }

            // check if any other tree is higher or has the same hight as the current tree
            let mut visible_from_left = true;
            let mut view_distance_left: u32 = u32::MAX;
            let mut visible_from_right = true;
            let mut view_distance_right: u32 = u32::MAX;
            for (other_x, other_tree) in row.iter().enumerate() {
                if other_x < x && other_tree >= tree {
                    visible_from_left = false;
                    view_distance_left = u32::min(view_distance_left, (x - other_x) as u32);
                }
                if other_x > x && other_tree >= tree {
                    visible_from_right = false;
                    view_distance_right = u32::min(view_distance_right, (other_x - x) as u32);
                }
            }
            // tree is on edge
            if view_distance_left == u32::MAX {
                view_distance_left = x as u32;
            }
            if view_distance_right == u32::MAX {
                view_distance_right = (row.len() - x) as u32 - 1;
            }

            let mut visible_from_top = true;
            let mut view_distance_top: u32 = u32::MAX;
            let mut visible_from_bottom = true;
            let mut view_distance_bottom: u32 = u32::MAX;
            for (other_y, other_tree) in
                grid.iter().enumerate().filter_map(|(other_y, other_row)| {
                    other_row.get(x).map(|other_tree| (other_y, other_tree))
                })
            {
                if other_y < y && other_tree >= tree {
                    visible_from_top = false;
                    view_distance_top = u32::min(view_distance_top, (y - other_y) as u32);
                }
                if other_y > y && other_tree >= tree {
                    visible_from_bottom = false;
                    view_distance_bottom = u32::min(view_distance_bottom, (other_y - y) as u32);
                }
            }
            // tree is on edge
            if view_distance_top == u32::MAX {
                view_distance_top = y as u32;
            }
            if view_distance_bottom == u32::MAX {
                view_distance_bottom = (grid.len() - y) as u32 - 1;
            }

            if visible_from_left || visible_from_right || visible_from_top || visible_from_bottom {
                visible_trees += 1;
            }
            let scenic_score =
                view_distance_left * view_distance_right * view_distance_top * view_distance_bottom;
            println!("[{tree}] {x}x{y} => {scenic_score} ({view_distance_left} * {view_distance_right} * {view_distance_top} * {view_distance_bottom})");

            if scenic_score > highest_scenic_score {
                highest_scenic_score = scenic_score;
                scenic_score_position = (x, y);
            }
        }
    }

    println!("visible_trees: {visible_trees}");
    println!("highest scenic score: {highest_scenic_score} {scenic_score_position:?}");
    Ok(())
}
