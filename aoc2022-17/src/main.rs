use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use itertools::Itertools;
use ndarray::{array, Array1, Array2, ArrayBase, Axis, Ix1, Ix2};

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

fn create_checkpoint_key(grid: &Grid, highest_rock: usize) -> [usize; 9] {
    [0; 9]
}

fn solve1(
    tower_size: usize,
    verbose: bool,
    jet_patterns: &Vec<Jet>,
    rock_forms: &Vec<ArrayBase<impl ndarray::Data<Elem = bool>, Ix2>>,
) -> usize {
    let max_height = tower_size * 4 + 10;
    let mut grid = Grid::default((max_height, 7));
    let mut highest_rock = max_height;
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

    let mut seen = HashMap::<[u64; 7], ArrayBase<ndarray::OwnedRepr<bool>, Ix2>>::new();

    while rock_count <= tower_size {
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
                highest_rock = highest_rock.min(y);
                if verbose {
                    println!("materialize!");
                }
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
        highest_rock,
        // find_highest_rock(&grid),
        max_height - highest_rock
    );

    max_height - highest_rock
}

fn is_height_colliding(
    heights: &ArrayBase<impl ndarray::Data<Elem = usize>, Ix1>,
    rock_bottom: &ArrayBase<impl ndarray::Data<Elem = usize>, Ix1>,
    x: usize,
    y: usize,
) -> bool {
    for x0 in 0..rock_bottom.len() {
        if heights[x + x0] > y - rock_bottom[x0] {
            return true;
        }
    }
    // for (x0, bottom) in rock_bottom.iter().enumerate() {
    //     // dbg!(heights[x + x0], bottom, y - bottom);
    //     if heights[x + x0] > y - bottom {
    //         return true;
    //     }
    // }
    false
}

fn solve2(
    tower_size: usize,
    verbose: bool,
    jet_patterns: &Vec<Jet>,
    rock_forms: &Vec<ArrayBase<impl ndarray::Data<Elem = bool>, Ix2>>,
) {
    let rock_top_heights = rock_forms
        .iter()
        .map(|rock| {
            rock.axis_iter(Axis(1))
                .map(|col| {
                    col.iter()
                        .enumerate()
                        .find(|(_, c)| **c)
                        .map_or(0, |(y, _)| y)
                })
                .collect::<Array1<_>>()
        })
        .collect::<Vec<_>>();
    let rock_bottom_heights = rock_forms
        .iter()
        .map(|rock| {
            let rock_height = rock.shape()[0];
            rock.axis_iter(Axis(1))
                .map(|col| {
                    col.iter()
                        .rev()
                        .enumerate()
                        .find(|(_, c)| **c)
                        .map_or(0, |(y, _)| rock_height - y)
                })
                .collect::<Array1<_>>()
        })
        .collect::<Vec<_>>();
    let rock_shapes = rock_forms
        .iter()
        .map(|rock| {
            let shape = rock.shape();
            (shape[0], shape[1])
        })
        .collect::<Vec<_>>();

    dbg!(&rock_top_heights, &rock_bottom_heights);

    let mut heights = Array1::<usize>::zeros(7);
    let mut highest_rock = 0;
    // let mut form_iterator = LoopingIterator::new(&(0..rock_forms.len()).collect::<Vec<usize>>());
    let mut rock_idx: usize = 0;
    let mut jet_pos: usize = 0;

    for n in 1..=tower_size {
        let rock_bottom = &rock_bottom_heights[rock_idx];
        // let rock_shape = (*rock_bottom.iter().max().unwrap(), rock_bottom.len());
        let rock_shape = &rock_shapes[rock_idx];
        let mut x: usize = 2;
        let mut y: usize = highest_rock + rock_shape.0 + 3;

        if verbose {
            println!("----");
            println!("Highest Rock: {highest_rock}");
            println!("New Rock #{rock_idx} {rock_shape:?} @({y}, {x})");
            draw_grid(&rock_forms[rock_idx]);
            println!();
        }

        loop {
            let jet = jet_patterns[jet_pos];
            jet_pos = (jet_pos + 1) % jet_patterns.len();

            match jet {
                Jet::Left if x > 0 && !is_height_colliding(&heights, rock_bottom, x - 1, y) => {
                    x -= 1;
                    if verbose {
                        println!("left!");
                    }
                }
                Jet::Right
                    if x + 1 <= 7 - rock_shape.1
                        && !is_height_colliding(&heights, rock_bottom, x + 1, y) =>
                {
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

            // dbg!(y, y + rock_shape.0);
            // check if the form can fall any further
            if y - rock_shape.0 == 0 || is_height_colliding(&heights, rock_bottom, x, y - 1) {
                // can't move any further
                if verbose {
                    println!("materialize! @({y}, {x})");
                    println!("{}", heights.iter().map(|h| format!("{h:06}")).join(" "));
                }

                let rock_top = &rock_top_heights[rock_idx];
                for (x0, top) in rock_top.iter().enumerate() {
                    // dbg!(x + x0, &heights[x + x0], y - top);
                    heights[x + x0] = y - top;
                    if heights[x + x0] > highest_rock {
                        highest_rock = heights[x + x0];
                    }
                }

                break;
            }

            // down, down, down
            y -= 1;
        }

        rock_idx = (rock_idx + 1) % rock_forms.len();

        if verbose {
            println!("{}", heights.iter().map(|h| format!("{h:06}")).join(" "));
        } else if n % 10000000 == 0 {
            println!(
                "{n} / {tower_size} ({:.2}%)",
                (n as f64 / tower_size as f64) * 100.
            )
        }
    }

    println!("{}", heights.iter().map(|h| format!("{h:06}")).join(" "));
    println!("Highest Rock: {highest_rock}");
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

    // let tower_size = 10;

    dbg!(
        rock_forms.len(),
        jet_patterns.len(),
        rock_forms.len() * jet_patterns.len()
    );

    solve1(
        2022,
        // tower_size,
        false,
        &jet_patterns,
        &rock_forms,
    );

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
