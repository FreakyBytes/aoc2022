use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vec2(usize, usize);

#[derive(Debug)]
enum Elevation {
    Start,
    End,
    Height(i64),
}

impl From<&char> for Elevation {
    fn from(value: &char) -> Self {
        match value {
            'S' => Self::Start,
            'E' => Self::End,
            d => Self::Height((*d as i64) - 97),
        }
    }
}

fn get_by_coordinate<'a>(
    grid: &'a Vec<Vec<Elevation>>,
    coordinates: Vec2,
) -> Option<&'a Elevation> {
    grid.get(coordinates.1 as usize)
        .map_or(None, |row| row.get(coordinates.0 as usize))
}

fn find_start_end(grid: &Vec<Vec<Elevation>>) -> (Option<Vec2>, Option<Vec2>) {
    let mut start: Option<Vec2> = None;
    let mut end: Option<Vec2> = None;

    for (x, row) in grid.iter().enumerate() {
        for (y, elevation) in row.iter().enumerate() {
            match elevation {
                Elevation::Start => start = Some(Vec2(x, y)),
                Elevation::End => end = Some(Vec2(x, y)),
                _ => {}
            }
        }
    }

    (start, end)
}

fn get_size(grid: &Vec<Vec<Elevation>>) -> Vec2 {
    Vec2(
        grid.iter().map(|row| row.len()).max().unwrap_or(0),
        grid.len(),
    )
}

fn patch_surrounding(
    grid: &Vec<Vec<Elevation>>,
    size: Vec2,
    distances: &mut HashMap<Vec2, u32>,
    coord: Vec2,
    elev: i64,
    dist: u32,
) -> bool {
    let mut changed_something = false;
    for (x0, y0) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let (x0, y0) = (coord.0 as i64 + x0, coord.1 as i64 + y0);
        if x0 < 0 || x0 >= size.0 as i64 || y0 < 0 || y0 >= size.1 as i64 {
            continue;
        }
        drop(x0);
        drop(y0);
        let coord0 = Vec2(x0 as usize, y0 as usize);
        let dist0 = dist + 1;

        match (
            get_by_coordinate(grid, coord0).unwrap(),
            distances.get(&coord0),
        ) {
            (Elevation::Start, _) => {}
            (Elevation::End, Some(dist0_old)) if elev >= 25 && *dist0_old > dist0 => {
                distances.insert(coord0, dist0);
                changed_something = true;

                println!("Inserted: {coord0:?} End dist={dist0:?}");
            }
            (Elevation::End, None) if elev >= 25 => {
                distances.insert(coord0, dist0);
                changed_something = true;

                println!("Inserted: {coord0:?} End dist={dist0:?}");
            }
            (Elevation::Height(elev0), Some(dist0_old))
                if *elev0 - elev <= 1 && *dist0_old > dist0 =>
            {
                distances.insert(coord0, dist0);
                changed_something = true;

                println!("Overwrote: {coord0:?} {elev0:?} dist={dist0:?}");
            }
            (Elevation::Height(elev0), None) if *elev0 - elev <= 1 => {
                distances.insert(coord0, dist0);
                changed_something = true;

                println!("Inserted: {coord0:?} {elev0:?} dist={dist0:?}");
            }
            _ => {}
        }
    }

    changed_something
}

fn iterate_distance_map(
    grid: &Vec<Vec<Elevation>>,
    size: Vec2,
    distances: &mut HashMap<Vec2, u32>,
) -> Option<u32> {
    // for (x, y) in (0..size.0).zip(0..size.1) {
    let mut changed_something = false;
    for x in 0..size.0 {
        for y in 0..size.1 {
            let coord = Vec2(x, y);
            let elev = get_by_coordinate(&grid, coord).unwrap();
            let dist = distances.get(&coord);
            // println!("{coord:?} {elev:?} dist={dist:?}");
            match (elev, dist) {
                (Elevation::Start, _) => {
                    distances.insert(coord, 0);
                    if patch_surrounding(grid, size, distances, coord, 0, 0) {
                        changed_something = true;
                    }
                }
                // (Elevation::Start, Some(_)) => {}
                (Elevation::End, Some(dist)) if changed_something == false && *dist < 10000 => {
                    return Some(*dist);
                }
                (Elevation::Height(0), None) => {
                    distances.insert(coord, 0);
                    changed_something = true;
                    println!("Inserted: {coord:?} {elev:?} dist=0");
                    if patch_surrounding(grid, size, distances, coord, 0, 0) {
                        changed_something = true;
                    }
                }
                (Elevation::Height(0), Some(dist)) if *dist != 0 => {
                    distances.insert(coord, 0);
                    changed_something = true;
                    println!("Inserted: {coord:?} {elev:?} dist=0");
                    if patch_surrounding(grid, size, distances, coord, 0, 0) {
                        changed_something = true;
                    }
                }
                (Elevation::Height(elev), Some(dist)) => {
                    if patch_surrounding(grid, size, distances, coord, *elev, *dist) {
                        changed_something = true;
                    }
                }
                _ => {}
            }
        }
    }

    if changed_something == false {
        // this iteration nothing changed, but we also did not reach the end
        panic!("No path to End found!");
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let grid = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| {
            line.chars()
                .map(|c| Elevation::from(&c))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let (start, end) = find_start_end(&grid);
    let start = start.expect("Found no start");
    let end = end.expect("Found no end");
    let size = get_size(&grid);
    println!("Size: {size:?}  |  Start: {start:?}  | end: {end:?}");

    // init distance map
    let mut distances: HashMap<Vec2, u32> = HashMap::with_capacity((size.0 * size.1) + 1);
    // let mut distances: HashMap<Vec2, u32> = (0..size.0)
    //     .zip(0..size.1)
    //     .map(|(x, y)| (Vec2(x, y), 10000))
    //     .collect()>;

    for _ in 0..1000 {
        let dist = iterate_distance_map(&grid, size, &mut distances);
        // println!("{distances:?}");
        println!();
        print!("     ");
        for x in 0..size.0 {
            print!("{:>3} ", x.to_string());
        }
        println!();
        for y in 0..size.1 {
            print!("{:>4} ", y.to_string());
            for x in 0..size.0 {
                let coord = Vec2(x, y);
                // let elev = get_by_coordinate(&grid, coord).unwrap();
                let dist = distances.get(&coord);
                // println!("{coord:?} {elev:?} dist={dist:?}");
                if let Some(dist) = dist {
                    print!("{:>3} ", dist.to_string());
                } else {
                    print!("  . ");
                }
            }
            println!();
        }
        println!("\n----\n");

        if let Some(dist) = dist {
            println!("final distance {dist}");
            break;
        }
    }

    Ok(())
}
