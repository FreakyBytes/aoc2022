use itertools::Itertools;
use std::{
    collections::HashSet,
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

    for line in lines {
        print!("{line}");
        let pos = line
            .chars()
            .tuple_windows::<(_, _, _, _)>()
            .find_position(|(a, b, c, d)| a != b && a != c && a != d && b != c && b != d && c != d);
        println!(
            "start of packet => {:?}",
            pos.map(|(p, chars)| (p + 4, chars))
        );

        for pos in 14..line.len() {
            let window = &line[(pos - 14)..pos];
            let unique_window: HashSet<char> = HashSet::from_iter(window.chars());
            if unique_window.len() == 14 {
                println!("start of message => {pos} {window}");
                break;
            }
        }
    }

    Ok(())
}
