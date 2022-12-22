use std::fs::read_to_string;

use anyhow::{Context, Result};
use pest::Parser;

#[derive(Debug, Parser)]
#[grammar = "map.pest"]
struct InputParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalkRule {
    Step(u32),
    TurnLeft,
    TurnRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    East = 0,
    South = 1,
    West = 2,
    North = 3,
}

impl Direction {
    pub fn turn_left(&self) -> Self {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    pub fn turn_right(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridCell {
    Void,
    Empty(Option<Direction>),
    Rock,
}

pub type Grid = Vec<Vec<GridCell>>;

pub fn parse_input(file_name: &str) -> Result<(Grid, Vec<WalkRule>)> {
    let input = read_to_string(file_name)?;
    let mut parsed = InputParser::parse(Rule::input, &input)?;
    let top_level = parsed.next().context("Couldn't parse top-level of input")?;

    let mut walk_rules: Vec<WalkRule> = Vec::new();
    let mut grid: Vec<Vec<GridCell>> = Vec::new();
    for p in top_level.into_inner() {
        match p.as_rule() {
            Rule::map_row => {
                let row = p
                    .into_inner()
                    .map(|t| match t.as_rule() {
                        Rule::VOID => GridCell::Void,
                        Rule::SPACE => GridCell::Empty(None),
                        Rule::ROCK => GridCell::Rock,
                        _ => panic!("Unexpected rule in map_row"),
                    })
                    .collect::<Vec<_>>();
                grid.push(row);
            }
            Rule::walk_rules => {
                p.into_inner()
                    .map(|t| match t.as_rule() {
                        Rule::step => WalkRule::Step(t.as_str().parse().unwrap()),
                        Rule::LEFT => WalkRule::TurnLeft,
                        Rule::RIGHT => WalkRule::TurnRight,
                        _ => panic!("Unexpected rule in walk_rules"),
                    })
                    .for_each(|wr| {
                        walk_rules.push(wr);
                    });
            }
            _ => panic!("Unexpected top-level rule"),
        };
    }

    Ok((grid, walk_rules))
}
