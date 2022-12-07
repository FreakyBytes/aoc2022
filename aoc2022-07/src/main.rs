use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use regex::Regex;

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let lines = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok());

    let rset = vec![
        // cd command
        r"^\$\s+cd\s+(?P<name>[a-z0-9\./]+)$",
        // ls command
        r"^\$\s+ls$",
        // dir response
        r"^dir\s+(?P<name>[a-z0-9\.]+)$",
        // file response
        r"^(?P<size>\d+)\s(?P<name>[a-z0-9\.]+)$",
    ]
    .iter()
    .map(|r| Regex::new(r).with_context(|| format!("Failed to compile regex '{r}'")))
    .collect::<Result<Vec<_>, _>>()?;

    let mut file_sizes: HashMap<Vec<String>, u64> = HashMap::new();
    let mut directories: HashSet<Vec<String>> = HashSet::new();
    let mut current_dir: Vec<String> = Vec::new();
    for line in lines {
        let Some((idx, caps)) = rset
            .iter()
            .enumerate()
            .filter_map(|(idx, r)| r.captures(&line).map(|caps| (idx, caps)))
            .take(1)
            .next() else { continue; };

        match (idx, caps.name("name").map(|g| g.as_str())) {
            // cd command
            (0, Some("/")) => {
                current_dir.clear();
            }
            (0, Some("..")) => {
                current_dir.pop();
            }
            (0, Some(name)) => current_dir.extend(name.split("/").map(|s| s.to_string())),
            // ls command
            (1, _) => {}
            // dir response
            (2, _) => {
                directories.insert(current_dir.clone());
            }
            // file response
            (3, Some(name)) => {
                directories.insert(current_dir.clone());

                let size = caps
                    .name("size")
                    .map(|s| s.as_str().parse::<u64>().unwrap())
                    .ok_or_else(|| anyhow::Error::msg("Failed to parse size!"))?;
                let mut path = current_dir.clone();
                path.push(name.to_string());
                file_sizes.insert(path, size);
            }
            _ => {
                panic!("Unknown line input! {idx} {caps:?} '{line}'")
            }
        }
    }
    println!("{directories:?}");
    println!("{file_sizes:?}");
    println!();

    let mut directory_sizes: HashMap<Vec<String>, u64> = HashMap::new();
    for path in directories.iter() {
        let dir_size: u64 = file_sizes
            .iter()
            .filter_map(|(file_path, size)| {
                if !path.iter().zip(file_path).all(|(a, b)| a == b) {
                    // path of file does not match
                    return None;
                }
                Some(size)
            })
            .sum();
        println!("{dir_size} {path:?}");
        directory_sizes.insert(path.clone(), dir_size);
    }
    let small_dir_total_size: u64 = directory_sizes
        .iter()
        .filter_map(|(_, size)| if *size < 100000 { Some(size) } else { None })
        .sum();
    println!("=> Total size of directories <100000 = {small_dir_total_size}");

    static TOTAL_SPACE: u64 = 70000000;
    static REQUIRED_UPDATE_SPACE: u64 = 30000000;
    let total_size = file_sizes.iter().map(|(_, size)| *size).sum::<u64>();
    let current_free_space = TOTAL_SPACE - total_size;
    // amount of space we need to free
    let required_space = REQUIRED_UPDATE_SPACE - current_free_space;
    let mut delete_candidates = directory_sizes
        .iter()
        .filter(|(_, size)| **size > required_space)
        .collect::<Vec<_>>();

    delete_candidates.sort_by(|(_, a), (_, b)| a.cmp(b));
    println!("Candidates to delete to make room for the update:");
    println!(
        "{}",
        delete_candidates
            .iter()
            .map(|(path, size)| format!("{size} => /{}\n", path.join("/")))
            .collect::<String>()
    );

    Ok(())
}
