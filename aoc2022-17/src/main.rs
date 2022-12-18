use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use itertools::Itertools;
use ndarray::{array, s, Array1, Array2, ArrayBase, Axis, Ix1, Ix2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Jet {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct LoopingIterator<'a, T> {
    pos: usize,
    slice: &'a [T],
}

impl<'a, T> LoopingIterator<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        Self { pos: 0, slice }
    }
}

impl<'a, T> Iterator for LoopingIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let res = &self.slice[self.pos];
        self.pos = (self.pos + 1) % self.slice.len();
        Some(res)
    }
}

type Grid = Array2<bool>;

fn draw_grid(grid: &ArrayBase<impl ndarray::Data<Elem = bool>, Ix2>) {
    let mut began = false;
    println!("     +0123456+");
    for (y, row) in grid.axis_iter(Axis(0)).enumerate() {
        if began || row.iter().any(|c| *c) {
            began = true;
            print!("{y:04} |");
            for cell in row {
                match cell {
                    true => print!("#"),
                    false => print!("."),
                };
            }
            println!("|");
        }
    }

    println!("     +-------+");
}

fn find_highest_rock(grid: &Grid) -> usize {
    // let height = *grid.shape().first().unwrap();
    // for y in 0..height {
    //     if grid[[0]]
    // }
    grid.axis_iter(Axis(0))
        .enumerate()
        .find(|(_y, row)| row.iter().any(|c| *c))
        .map_or_else(|| grid.shape()[0], |(y, _)| y)
}

fn is_colliding(
    grid: &Grid,
    rock: &ArrayBase<impl ndarray::Data<Elem = bool>, Ix2>,
    x: usize,
    y: usize,
) -> bool {
    let rock_shape = rock.shape();
    // println!(
    //     "stencil @({y}..{}, {x}..{})",
    //     y + rock_shape[0],
    //     x + rock_shape[1]
    // );
    // let stencil = grid.slice(s![y..(y + rock_shape[0]), x..(x + rock_shape[1])]);
    // draw_grid(&stencil);
    // rock == stencil

    for (y0, x0) in (0..rock_shape[0]).cartesian_product(0..rock_shape[1]) {
        if rock[[y0, x0]] && grid[[y + y0, x + x0]] {
            return true;
        }
    }
    false
}

fn materialize_rock(
    grid: &mut Grid,
    rock: &ArrayBase<impl ndarray::Data<Elem = bool>, Ix2>,
    x: usize,
    y: usize,
) {
    let rock_shape = rock.shape();
    for (y0, x0) in (0..rock_shape[0]).cartesian_product(0..rock_shape[1]) {
        if rock[[y0, x0]] {
            grid[[y + y0, x + x0]] = true;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Checkpoint {
    rock_idx: usize,
    jet_idx: usize,
    top_rocks: usize,
}

fn create_checkpoint_key(
    grid: &Grid,
    highest_rock: usize,
    rock_idx: usize,
    jet_idx: usize,
    look_back: usize,
    max_height: usize,
) -> Checkpoint {
    let mut tops = [look_back; 7];

    for x in 0..7 {
        for y in highest_rock..=(highest_rock + look_back).min(max_height - 1) {
            if grid[[y, x]] {
                tops[x] = tops[x].min(y - highest_rock)
            }
        }
    }

    // fingerprint of top rocks should be sufficient and even better! ;-)
    let mut top_rocks = 0;
    for x in 0..7 {
        top_rocks += 10_usize.pow(x) * (max_height - tops[x as usize]);
    }

    Checkpoint {
        rock_idx,
        jet_idx,
        top_rocks,
    }
}

fn solve1(
    num_rocks_to_drop: usize,
    verbose: bool,
    jet_patterns: &Vec<Jet>,
    rock_forms: &Vec<ArrayBase<impl ndarray::Data<Elem = bool>, Ix2>>,
) -> usize {
    let look_back: usize = 1000;
    let max_height = (num_rocks_to_drop * 4 + 10).min(2022 * 8 + 10);
    let mut grid = Grid::default((max_height, 7));
    let mut highest_rock = max_height;
    let mut final_height = 0;
    // let mut form_iterator = LoopingIterator::new(rock_forms);
    // let mut jet_iterator = LoopingIterator::new(&jet_patterns);
    let mut rock_idx: usize = 0;
    let mut jet_idx: usize = 0;
    let mut rock_count: usize = 1;

    // let rock = rock_forms.get(1).unwrap();
    // dbg!(is_colliding(&grid, rock, 0, max_height - 10));
    // materialize_rock(&mut grid, rock, 0, max_height - 10);
    // draw_grid(&grid);
    // dbg!(is_colliding(&grid, rock, 0, max_height - 10));

    let mut seen = HashMap::<Checkpoint, (usize, usize)>::new();

    while rock_count <= num_rocks_to_drop {
        let rock = &rock_forms[rock_idx];
        let rock_shape = rock.shape();
        let rock_shape = (rock_shape[0], rock_shape[1]);
        // let highest_rock = find_highest_rock(&grid);
        let mut x: usize = 2;
        let mut y: usize = highest_rock - rock_shape.0 - 3;

        if verbose {
            println!("----");
            println!("Highest Rock: {highest_rock}");
            println!("New Rock: {rock_shape:?} @({y}, {x})");
            draw_grid(rock);
            println!();
        }

        loop {
            let jet = jet_patterns[jet_idx];
            jet_idx = (jet_idx + 1) % jet_patterns.len();

            match jet {
                Jet::Left if x > 0 && !is_colliding(&grid, rock, x - 1, y) => {
                    x -= 1;
                    if verbose {
                        println!("left!");
                    }
                }
                Jet::Right if x + 1 <= 7 - rock_shape.1 && !is_colliding(&grid, rock, x + 1, y) => {
                    x += 1;
                    if verbose {
                        println!("right!");
                    }
                }
                _ => {
                    if verbose {
                        println!("{jet:?} - but no action");
                    }
                }
            }

            // check if the form can fall any further
            if y + rock_shape.0 >= max_height || is_colliding(&grid, rock, x, y + 1) {
                // can't move any further
                materialize_rock(&mut grid, rock, x, y);
                rock_count += 1;
                let offset = highest_rock - highest_rock.min(y);
                highest_rock -= offset;
                final_height += offset;
                if verbose {
                    println!("materialize!");
                }

                if rock_count >= num_rocks_to_drop {
                    break;
                }

                let cp = create_checkpoint_key(
                    &grid,
                    highest_rock,
                    rock_idx,
                    jet_idx,
                    look_back,
                    max_height,
                );
                if let Some((cp_rock_count, cp_final_height)) = seen.get(&cp) {
                    let remaining = num_rocks_to_drop - rock_count;
                    let possible_repetition: usize = remaining / (rock_count - cp_rock_count);
                    // let possible_repetition =
                    //     num::Integer::div_floor(&remaining, &(rock_count - cp_rock_count));
                    let offset = possible_repetition * (final_height - cp_final_height);
                    // dbg!(
                    //     num_rocks_to_drop,
                    //     rock_count,
                    //     remaining,
                    //     rock_count - cp_rock_count,
                    //     offset
                    // );
                    let rocks_to_skip = possible_repetition * (rock_count - cp_rock_count);
                    println!(
                        "Found checkpoint after {rock_count} rocks, possible reps: {possible_repetition}, skipping {rocks_to_skip}"
                    );

                    rock_count += rocks_to_skip;
                    final_height += offset;
                    seen.clear();
                }
                seen.insert(cp, (rock_count, final_height));
                break;
            }

            // down, down, down
            y += 1;
        }

        rock_idx = (rock_idx + 1) % rock_forms.len();

        if verbose {
            println!();
            draw_grid(&grid);
        }
    }

    println!();
    // draw_grid(&grid);
    println!();
    dbg!(
        max_height,
        highest_rock,
        max_height - highest_rock,
        final_height,
        // max_height - final_height,
    );

    // max_height - final_height
    final_height
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let jet_patterns = BufReader::new(
        File::open(file_name).with_context(|| "Could not open file: '{file_name}'")?,
    )
    .lines()
    .into_iter()
    .filter_map(|line| line.ok())
    .flat_map(|line| line.chars().collect::<Vec<_>>())
    .map(|c| match c {
        '<' => Jet::Left,
        '>' => Jet::Right,
        _ => panic!("Unknown char {c}"),
    })
    .collect::<Vec<_>>();

    //
    // EVERYTHING is (Y, X) !!
    //

    let rock_forms = vec![
        // ####
        array![[true, true, true, true]],
        // .#.
        // ###
        // .#.
        array![
            [false, true, false],
            [true, true, true],
            [false, true, false]
        ],
        // ..#
        // ..#
        // ###
        array![
            [false, false, true],
            [false, false, true],
            [true, true, true],
        ],
        // #
        // #
        // #
        // #
        array![[true], [true], [true], [true],],
        // ##
        // ##
        array![[true, true], [true, true],],
    ];

    let part1 = solve1(2022, false, &jet_patterns, &rock_forms);
    println!("Solution for part 1: {part1} ({})", part1 == 3068);

    println!("\n\n----------------------------------------\n\n");

    let part2 = solve1(1000000000000, false, &jet_patterns, &rock_forms);
    println!("Solution for part 2: {part2} ({})", part2 == 1514285714288);

    // let kgv = rock_forms.len() * jet_patterns.len();
    // let height_for_kgv = solve1(kgv, false, &jet_patterns, &rock_forms);
    // let result = (1000000000000_f64 / kgv as f64) * height_for_kgv as f64;
    // dbg!(kgv, height_for_kgv, result);
    // dbg!(1514285714288_f64 - result);

    // println!("\n****\n");
    // solve2(
    //     // 1000000000000,
    //     tower_size,
    //     // 2022,
    //     false,
    //     &jet_patterns,
    //     &rock_forms,
    // );

    Ok(())
}
