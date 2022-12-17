use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use itertools::Itertools;
use ndarray::{array, Array2, ArrayBase, Axis, Ix2};

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

fn is_colliding(grid: &Grid, rock: &Grid, x: usize, y: usize) -> bool {
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

fn materialize_rock(grid: &mut Grid, rock: &Grid, x: usize, y: usize) {
    let rock_shape = rock.shape();
    for (y0, x0) in (0..rock_shape[0]).cartesian_product(0..rock_shape[1]) {
        if rock[[y0, x0]] {
            grid[[y + y0, x + x0]] = true;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let jet_patterns = BufReader::new(File::open(file_name)?)
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

    let max_height = 2022 * 4 + 10;
    let verbose = false;
    let mut grid = Grid::default((max_height, 7));
    let mut form_iterator = LoopingIterator::new(&rock_forms);
    // let mut jet_iterator = LoopingIterator::new(&jet_patterns);
    let mut jet_pos = 0;

    // let rock = rock_forms.get(1).unwrap();
    // dbg!(is_colliding(&grid, rock, 0, max_height - 10));
    // materialize_rock(&mut grid, rock, 0, max_height - 10);
    // draw_grid(&grid);
    // dbg!(is_colliding(&grid, rock, 0, max_height - 10));

    // return Ok(());

    for _ in 1..=2022 {
        // for _ in 1..=5 {
        let rock = form_iterator.next().unwrap();
        let rock_shape = rock.shape();
        let rock_shape = (rock_shape[0], rock_shape[1]);
        let highest_rock = find_highest_rock(&grid);
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
            let jet = jet_patterns.get(jet_pos).unwrap();
            jet_pos = (jet_pos + 1) % jet_patterns.len();

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
                if verbose {
                    println!("materialize!");
                }
                break;
            }

            // down, down, down
            y += 1;
        }

        if verbose {
            println!();
            draw_grid(&grid);
        }
    }

    println!();
    draw_grid(&grid);
    println!();
    let highest_rock = find_highest_rock(&grid);
    dbg!(highest_rock, max_height - highest_rock);

    Ok(())
}
