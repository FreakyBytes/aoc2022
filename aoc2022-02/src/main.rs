use std::{
    cmp::Ordering,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

const SCORE_LOST: u32 = 0;
const SCORE_DRAW: u32 = 3;
const SCORE_WIN: u32 = 6;

#[derive(PartialEq, Eq, Ord, Debug, Clone)]
enum HandSign {
    ROCK,
    PAPER,
    SCISSOR,
}

impl HandSign {
    pub fn to_score(&self) -> u32 {
        match self {
            Self::ROCK => 1,
            Self::PAPER => 2,
            Self::SCISSOR => 3,
        }
    }
}

impl TryFrom<&str> for HandSign {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A" | "X" => Ok(Self::ROCK),
            "B" | "Y" => Ok(Self::PAPER),
            "C" | "Z" => Ok(Self::SCISSOR),
            _ => Err(anyhow::Error::msg(format!(
                "Failed to parse '{}' into handsign!",
                value
            ))),
        }
    }
}

impl PartialOrd for HandSign {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self == &Self::ROCK && other == &Self::PAPER {
            Some(Ordering::Less)
        } else if self == &Self::PAPER && other == &Self::SCISSOR {
            Some(Ordering::Less)
        } else if self == &Self::SCISSOR && other == &Self::ROCK {
            Some(Ordering::Less)
        } else if self == &Self::PAPER && other == &Self::ROCK {
            Some(Ordering::Greater)
        } else if self == &Self::ROCK && other == &Self::SCISSOR {
            Some(Ordering::Greater)
        } else if self == &Self::SCISSOR && other == &Self::PAPER {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WinningInstructions {
    LOSE,
    DRAW,
    WIN,
}

impl TryFrom<&str> for WinningInstructions {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "X" => Ok(Self::LOSE),
            "Y" => Ok(Self::DRAW),
            "Z" => Ok(Self::WIN),
            _ => Err(anyhow::Error::msg(format!(
                "Failed to parse '{}' into winning instruction!",
                value
            ))),
        }
    }
}

impl WinningInstructions {
    pub fn to_handsign(&self, opponent: &HandSign) -> HandSign {
        match self {
            Self::DRAW => opponent.clone(),
            Self::LOSE => match opponent {
                HandSign::ROCK => HandSign::SCISSOR,
                HandSign::PAPER => HandSign::ROCK,
                HandSign::SCISSOR => HandSign::PAPER,
            },
            Self::WIN => match opponent {
                HandSign::ROCK => HandSign::PAPER,
                HandSign::PAPER => HandSign::SCISSOR,
                HandSign::SCISSOR => HandSign::ROCK,
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let reader = BufReader::new(File::open(file_name)?);

    let mut total_score: u32 = 0;
    for line in reader.lines() {
        let (opponent, me): (HandSign, WinningInstructions) = {
            let line = line?;
            let mut split = line.splitn(2, " ");
            (
                split
                    .next()
                    .ok_or(anyhow::Error::msg("Insufficient elements"))?
                    .try_into()?,
                split
                    .next()
                    .ok_or(anyhow::Error::msg("Insufficient elements"))?
                    .try_into()?,
            )
        };
        let me = me.to_handsign(&opponent);

        let mut score: u32 = me.to_score();
        if me > opponent {
            score += SCORE_WIN;
        } else if me < opponent {
            score += SCORE_LOST;
        } else {
            score += SCORE_DRAW;
        }
        total_score += score;
        println!("{:?} {:?} {:?} {}", opponent, me, me > opponent, score);
    }
    println!("{}", total_score);

    Ok(())
}
