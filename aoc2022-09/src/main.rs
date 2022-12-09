use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader},
    ops::AddAssign,
    str::FromStr,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct Position2D {
    x: i64,
    y: i64,
}

impl Position2D {
    fn is_adjacent_to(&self, other: &Self) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx >= -1 && dx <= 1 && dy >= -1 && dy <= 1
    }

    fn apply_step_with_length_one(&mut self, step: &Step) {
        match step {
            Step::Up(_) => self.y -= 1,
            Step::Down(_) => self.y += 1,
            Step::Left(_) => self.x -= 1,
            Step::Right(_) => self.x += 1,
        }
    }
}

// impl Add<&Step> for Position2D {
//     type Output = Self;

//     fn add(self, rhs: &Step) -> Self::Output {
//         match rhs {
//             Step::Up(length) => Self {
//                 x: self.x,
//                 y: self.y - (*length as i64),
//             },
//             Step::Down(length) => Self {
//                 x: self.x,
//                 y: self.y + (*length as i64),
//             },
//             Step::Left(length) => Self {
//                 x: self.x - (*length as i64),
//                 y: self.y,
//             },
//             Step::Right(length) => Self {
//                 x: self.x + (*length as i64),
//                 y: self.y,
//             },
//         };

//         self
//     }
// }

impl AddAssign<&Step> for Position2D {
    fn add_assign(&mut self, rhs: &Step) {
        match rhs {
            Step::Up(length) => self.y -= *length as i64,
            Step::Down(length) => self.y += *length as i64,
            Step::Left(length) => self.x -= *length as i64,
            Step::Right(length) => self.x += *length as i64,
        }
    }
}

#[derive(Debug)]
enum Step {
    Up(u32),
    Down(u32),
    Left(u32),
    Right(u32),
}

impl Step {
    fn get_length(&self) -> u32 {
        match self {
            Self::Up(length) => *length,
            Self::Down(length) => *length,
            Self::Left(length) => *length,
            Self::Right(length) => *length,
        }
    }
}

impl FromStr for Step {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(" ") {
            Some(("U", length)) => Ok(Self::Up(length.parse()?)),
            Some(("D", length)) => Ok(Self::Down(length.parse()?)),
            Some(("L", length)) => Ok(Self::Left(length.parse()?)),
            Some(("R", length)) => Ok(Self::Right(length.parse()?)),
            None => Err(Self::Err::msg("Failed to split input!")),
            _ => Err(Self::Err::msg("Misformed input!")),
        }
    }
}

fn shift_knot_buffer(knots: &mut [Position2D]) {
    for idx in 1..knots.len() - 1 {
        knots[idx - 1] = knots[idx].clone();
    }
}

fn add_knots_to_tracker(positions: &mut HashSet<Position2D>, knots: &[Position2D]) {
    for idx in 0..knots.len() {
        positions.insert(knots[idx].clone());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let steps = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| Step::from_str(&line))
        .collect::<Result<Vec<_>, _>>()?;

    let mut tail_positions = HashSet::<Position2D>::new();
    let mut head = Position2D::default();
    let mut knots: [Position2D; 11] = [
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
        head.clone(),
    ];
    // let mut tail = head.clone();
    tail_positions.insert(head.clone());

    for step in steps.iter() {
        println!("==== {step:?} ====");
        for _ in 0..step.get_length() {
            let prev_head = head.clone();
            head.apply_step_with_length_one(step);
            if !head.is_adjacent_to(&knots[knots.len() - 1]) {
                shift_knot_buffer(&mut knots);
                knots[knots.len() - 1] = prev_head;
                add_knots_to_tracker(&mut tail_positions, &knots);
                // tail_positions.insert(tail.clone());
                // println!("na");
            }
            println!("head: {head:?}   | knots: {knots:?}");
        }
    }

    // println!("{tail_positions:?}");
    println!("Number of tail positions: {}", tail_positions.len());

    Ok(())
}
