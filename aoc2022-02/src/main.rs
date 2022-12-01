use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let reader = BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }

    Ok(())
}
