use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn score_from_char(input: &char) -> u32 {
    if input.is_ascii_lowercase() {
        (*input as u32) - 96
    } else if input.is_ascii_uppercase() {
        (*input as u32) - 64 + 26
    } else {
        panic!("'{}' is not an ascii character!", input);
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let reader = BufReader::new(File::open(file_name)?);

    let mut total_score: u32 = 0;
    for line in reader.lines() {
        let line = line?;
        let (comp1, comp2) = line.split_at(line.len() / 2);
        let comp1: HashSet<char> = HashSet::from_iter(comp1.chars());
        let comp2: HashSet<char> = HashSet::from_iter(comp2.chars());

        println!("{}:{:?} {}:{:?}", comp1.len(), comp1, comp2.len(), comp2);
        if let Some(intersect) = comp1.intersection(&comp2).next() {
            let score = score_from_char(intersect);
            total_score += score;
            println!(" -> {} {}", intersect, score);
        }
    }

    println!("==> {}", total_score);

    Ok(())
}
