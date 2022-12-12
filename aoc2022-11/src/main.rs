use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::Error;
use chumsky::Parser;
use parser::{Expr, MonkeyAction, MonkeyBool, MonkeyLang, MonkeyTestCondition};
use rug::Integer;

use crate::parser::{monkey_parser, print_parser_error};

mod parser;

#[derive(Debug, Default, Clone)]
struct Monkey {
    id: u32,
    items: VecDeque<Integer>,
    activity: u64,
    operation_expr: Expr,
    divisible_by: u32,
    target_if_true: u32,
    target_if_false: u32,
}

impl TryFrom<&MonkeyLang> for Monkey {
    type Error = Error;

    fn try_from(value: &MonkeyLang) -> Result<Self, Self::Error> {
        if let MonkeyLang::MonkeyDefinition(id, items) = value {
            let mut monkey = Self {
                id: *id,
                ..Default::default()
            };

            for item in items.iter() {
                match item {
                    MonkeyLang::MonkeyDefinition(_, _) => {
                        return Err(Self::Error::msg(
                            "Nested MonkeyDefinitions are not supported!",
                        ))
                    }
                    MonkeyLang::StartingItems(si) => {
                        monkey.items = VecDeque::from_iter(si.iter().map(|i| Integer::from(*i)));
                    }
                    MonkeyLang::Operation(expr) => monkey.operation_expr = expr.to_owned(),
                    MonkeyLang::Test {
                        divisible_by,
                        conditions,
                    } => {
                        monkey.divisible_by = *divisible_by;
                        for cond in conditions.iter() {
                            match cond {
                                MonkeyTestCondition(
                                    MonkeyBool::True,
                                    MonkeyAction::ThrowToMonkey(target),
                                ) => monkey.target_if_true = *target,
                                MonkeyTestCondition(
                                    MonkeyBool::False,
                                    MonkeyAction::ThrowToMonkey(target),
                                ) => monkey.target_if_false = *target,
                                // _ => {
                                //     return Err(Self::Error::msg(format!(
                                //         "Unknown MonkeyTestCondition! `{cond:?}`"
                                //     )));
                                // }
                            }
                        }
                    }
                }
            }

            Ok(monkey)
        } else {
            Err(Self::Error::msg(format!(
                "Value is not a MonkeyDefinition: `{value:?}`"
            )))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::env::args().nth(1).expect("No input file supplied!");
    let input = std::fs::read_to_string(file_name)?;
    //     let input = r#"
    // Monkey 0:
    //   Starting items: 12, 14
    //   Operation: new = old + 1
    //   Test: divisible by 12
    //     If true: throw to monkey 0
    //     If false: throw to monkey 1

    // "#
    //     .to_string();

    for (idx, line) in input.lines().enumerate() {
        println!("{idx:02}: {line}", idx = idx + 1);
    }
    println!();

    let tokens = monkey_parser().parse(input.clone()).map_err(|err| {
        err.into_iter().for_each(|e| {
            println!();
            // println!("{e:?}");
            print_parser_error(&input, e);
            println!();
        });
        anyhow::Error::msg("Failed to parse input!")
    })?;
    println!("\n{tokens:#?}");
    let mut monkeys = tokens
        .iter()
        .map(|t| -> Result<(u32, Arc<Mutex<Monkey>>), Error> {
            let m: Monkey = t.try_into()?;
            Ok((m.id, Arc::new(Mutex::new(m))))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    // monkeys.sort_by(|a, b| a.id.cmp(&b.id));

    println!("\n{monkeys:#?}");
    let sorted_keys = {
        let mut keys = monkeys.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    };

    // let mut monkey_activity: HashMap<u32, u32> =
    //     sorted_keys.iter().map(|idx| (*idx, 0_u32)).collect();

    const TOTAL_ROUNDS: i32 = 10_000;
    let start = Instant::now();
    let mut last_report = Instant::now();
    for round in 1..=TOTAL_ROUNDS {
        //     println!("==== Round {round:02} ====");
        //     println!();
        let round_start = Instant::now();

        for idx in sorted_keys.iter() {
            // println!("Monkey {idx}:");
            let mut monkey = monkeys
                .get(idx)
                .unwrap()
                .try_lock()
                .map_err(|_| Error::msg("Failed to acquire mutex lock for source monkey"))?;

            while let Some(item) = monkey.items.pop_front() {
                // println!("  Monkey inspects an item with worry level of {item}");
                monkey.activity += 1;
                let worry_level = monkey.operation_expr.eval_big(&item);
                // println!("    Applying expression, new worry level is {worry_level}");
                // worry_level /= 3;
                // println!(
                //     "    Monkey gets bored with item. Worry level is divided by 3 to {worry_level}"
                // );

                let divisible_by = monkey.divisible_by;
                let target = if worry_level.is_divisible_u(divisible_by) {
                    // println!(
                    //     "    Item with worry level {worry_level} is dividable by {divisible_by}"
                    // );
                    monkey.target_if_true
                } else {
                    //     println!(
                    //     "    Item with worry level {worry_level} is _not_ dividable by {divisible_by}"
                    // );
                    monkey.target_if_false
                };

                // println!("    Item with worry level {worry_level} is thrown to monkey {target}");
                if target == *idx {
                    monkey.items.push_back(worry_level);
                } else {
                    let mut target_monkey =
                        monkeys.get(&target).unwrap().try_lock().map_err(|_| {
                            Error::msg("Failed to acquire mutex lock for target monkey")
                        })?;
                    target_monkey.items.push_back(worry_level);
                }
            }
            // println!();
        }

        let elapsed = (Instant::now() - start).as_secs_f32();
        if round == 1 || round == 20 || round % 1000 == 0 {
            println!("==== Round {round:02} ====");
            for idx in sorted_keys.iter() {
                let monkey = monkeys.get(idx).unwrap().try_lock().map_err(|_| {
                    Error::msg("Failed to acquire mutex lock for counting monkey activity")
                })?;
                println!(
                    "Monkey {idx} inspected items {activity} times.",
                    activity = monkey.activity
                );
            }
        }
        if round == 1
            || round == 20
            || round % 100 == 0
            || (Instant::now() - last_report).as_secs() > 15
        {
            let round_duration = (Instant::now() - round_start).as_secs_f32();
            let avg = elapsed / round as f32;
            let eta = (TOTAL_ROUNDS - round) as f32 * avg;
            last_report = Instant::now();
            println!("Round {round:03} took {round_duration:.4}s | Total Elapsed: {elapsed:.4}s | Avg per Round {avg:.4}s | ETA {eta:.1}s (aka {eta_h:.2}h)", eta_h = eta / 3600.0);
            println!();
        }
    }

    let mut activity_rank = sorted_keys
        .iter()
        .map(|idx| {
            let monkey = monkeys
                .get(idx)
                .unwrap()
                .try_lock()
                .map_err(|_| Error::msg("Failed to acquire mutex lock for monkey"))?;

            Ok((*idx, monkey.activity))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    activity_rank.sort_by(|(_, a), (_, b)| b.cmp(a));
    for (idx, activity) in activity_rank.iter() {
        println!("Monkey {idx} inspected items {activity} times.")
    }

    let monkey_business_level: u64 = activity_rank
        .iter()
        .take(2)
        .map(|(_, a)| *a)
        .reduce(|a, b| a * b)
        .unwrap();
    println!("Level of Monkey Business: {monkey_business_level}");

    Ok(())
}
