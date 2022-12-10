use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Noop,
    AddX(i32),
}

impl FromStr for Op {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(' ') {
            Some(("noop", _)) => Ok(Self::Noop),
            Some(("addx", value)) => Ok(Self::AddX(value.parse()?)),
            None if s == "noop" => Ok(Self::Noop),
            None => Err(Self::Err::msg(format!(
                "Failed to split instruction! '{s}'",
            ))),
            _ => Err(Self::Err::msg(format!(
                "Failed to parse instruction! '{s}'"
            ))),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let ops = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok())
        .map(|line| Op::from_str(&line))
        .collect::<Result<Vec<_>, _>>()?;

    // let mut cycle: u32 = 1;
    let mut register_x: i32 = 1;
    let mut x_over_time = Vec::<i32>::new();
    for op in ops.iter() {
        // println!("{op:?}");
        match op {
            Op::Noop => {
                x_over_time.push(register_x);
            }
            Op::AddX(value) => {
                // takes 2 cycles
                x_over_time.push(register_x);
                x_over_time.push(register_x);
                register_x += *value;
            }
        }
        println!(
            "{:<15}  => CLK={:03} X={}",
            format!("{:?}", op),
            x_over_time.len(),
            register_x
        );
    }

    let interesting_cycles: [usize; 6] = [20, 60, 100, 140, 180, 220];
    let signal_strength = interesting_cycles
        .iter()
        .map(|idx| (*idx as i32) * x_over_time[*idx - 1])
        .collect::<Vec<_>>();
    // .sum::<i32>();

    println!();
    println!("signal strengths: {signal_strength:?}");
    println!(
        "signal strength sum: {}",
        signal_strength.iter().sum::<i32>()
    );

    println!();
    for idx in 0..240 {
        let x = x_over_time[idx];
        let col = (idx % 40) as i32;

        if (x - col).abs() <= 1 {
            print!("#");
        } else {
            print!(".");
        }
        if col == 39 {
            println!();
        }
    }

    Ok(())
}
