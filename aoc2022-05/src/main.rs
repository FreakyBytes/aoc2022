use anyhow::Context;
use itertools::Itertools;
use std::{
    collections::VecDeque,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
struct Stack {
    stack: VecDeque<char>,
}

impl Stack {
    fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    pub fn push(&mut self, crate_id: char) {
        self.stack.push_back(crate_id);
    }

    fn push_many(&mut self, lifted_crates: &mut VecDeque<char>) {
        for crate_id in lifted_crates.into_iter().rev() {
            self.stack.push_back(*crate_id);
        }
        // self.stack.append(lifted_crates);
    }

    fn take(&mut self) -> Option<char> {
        self.stack.pop_back()
    }

    fn take_many(&mut self, amount: u32) -> Result<VecDeque<char>, anyhow::Error> {
        (0..amount)
            .into_iter()
            .map(|_| self.stack.pop_back())
            .collect::<Option<_>>()
            .ok_or(anyhow::Error::msg("Not enough crates on stack!"))
    }

    fn top(&self) -> Option<char> {
        self.stack.back().copied()
    }
}

#[derive(Debug)]
struct Stacks {
    stacks: Vec<Stack>,
}

impl Stacks {
    fn new(capacity: usize) -> Self {
        let mut stacks = Vec::<Stack>::with_capacity(capacity);
        for _ in 0..capacity {
            stacks.push(Stack::new());
        }

        Self { stacks }
    }

    // fn get(&self, index: usize) -> Result<&Stack, anyhow::Error> {
    //     self.stacks
    //         .get(index)
    //         .with_context(|| anyhow::Error::msg(format!("There is no stack with index {}", index)))
    // }

    fn push_onto(&mut self, stack_index: usize, crate_id: char) -> Result<(), anyhow::Error> {
        self.stacks
            .get_mut(stack_index)
            .with_context(|| {
                anyhow::Error::msg(format!("There is no stack with index {}", stack_index))
            })?
            .push(crate_id);
        Ok(())
    }

    fn move_crates(&mut self, amount: u32, from: usize, to: usize) -> Result<(), anyhow::Error> {
        let (from, to) = (from - 1, to - 1);
        for _idx in 0..amount {
            let crate_id = self
                .stacks
                .get_mut(from)
                .with_context(|| {
                    anyhow::Error::msg(format!("There is no stack with index {from}"))
                })?
                .take()
                .ok_or_else(|| anyhow::Error::msg(format!("Stack {from} has no more crates!")))?;
            self.stacks
                .get_mut(to)
                .with_context(|| anyhow::Error::msg(format!("There is no stack with index {to}")))?
                .push(crate_id);
        }
        Ok(())
    }

    fn move_crates9001(
        &mut self,
        amount: u32,
        from: usize,
        to: usize,
    ) -> Result<(), anyhow::Error> {
        let (from, to) = (from - 1, to - 1);
        let mut crates = self
            .stacks
            .get_mut(from)
            .with_context(|| anyhow::Error::msg(format!("There is no stack with index {from}")))?
            .take_many(amount)?;
        println!("moving {crates:?}");
        self.stacks
            .get_mut(to)
            .with_context(|| anyhow::Error::msg(format!("There is no stack with index {to}")))?
            .push_many(&mut crates);
        Ok(())
    }

    fn get_tops(&self) -> Vec<char> {
        self.stacks.iter().filter_map(Stack::top).collect()
    }
}

fn parse_stack_drawing(drawing: &Vec<String>) -> Result<Stacks, anyhow::Error> {
    let number_of_stacks = drawing
        .last()
        .ok_or(anyhow::Error::msg("No stack indices"))?
        .split_whitespace()
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
        .count();

    let mut stacks = Stacks::new(number_of_stacks);
    for line in drawing.iter().rev().skip(1) {
        for idx in 0..number_of_stacks {
            // .with_context(|| {
            //     anyhow::Error::msg(format!("stack drawing too short (idx={idx}): {line}"))
            // })?;
            match line.chars().nth((idx * 4) + 1) {
                Some(crate_id) if crate_id != ' ' => stacks.push_onto(idx, crate_id)?,
                _ => {}
            }
        }
    }

    println!("Parsed Stacks: {stacks:?}");
    Ok(stacks)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let lines = BufReader::new(File::open(file_name)?)
        .lines()
        .into_iter()
        .filter_map(|line| line.ok());

    let move_regex =
        regex::Regex::new(r#"^move\s+(?P<amount>\d+)\s+from\s+(?P<from>\d+)\s+to\s+(?P<to>\d+)$"#)?;

    let mut stack_drawing = Vec::<String>::new();
    let mut stacks: Option<Stacks> = None;
    for line in lines {
        if line.is_empty() {
            stacks = Some(parse_stack_drawing(&stack_drawing)?);
            println!("{stack_drawing:?}");
            continue;
        } else if stacks.is_none() {
            stack_drawing.push(line);
            // println!("stack: {line}");
        } else if let Some(caps) = move_regex.captures(&line) {
            // println!("move: {line} -> {caps:?}");
            match (caps.name("amount"), caps.name("from"), caps.name("to")) {
                (Some(amount), Some(from), Some(to)) => {
                    //
                    stacks.as_mut().unwrap().move_crates9001(
                        amount.as_str().parse()?,
                        from.as_str().parse()?,
                        to.as_str().parse()?,
                    )?
                }
                _ => {}
            }
        }
    }

    let stacks = stacks.unwrap();
    let tops = stacks.get_tops();
    println!("{stacks:?}");
    println!("{:?}", tops.iter().map(char::to_string).join(""));
    Ok(())
}
