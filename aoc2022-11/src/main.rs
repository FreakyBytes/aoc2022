use std::error::Error;

use chumsky::Parser;
use parser::Expr;

use crate::parser::{monkey_parser, print_parser_error};

mod parser;

#[derive(Debug, Default, Clone)]
struct Monkey {
    starting_items: Vec<u32>,
    operation_str: Expr,
    divisible_by: u32,
    target_if_true: usize,
    target_if_false: usize,
}

fn eval_expr(expr: &Expr, old: i64) -> i64 {
    match expr {
        Expr::Num(val) => *val,
        Expr::Add(a, b) => eval_expr(a, old) + eval_expr(b, old),
        Expr::Mul(a, b) => eval_expr(a, old) * eval_expr(b, old),
        Expr::Assign(lhs, rhs) if Expr::New == **lhs => eval_expr(rhs, old),
        // Expr::Assign(Expr::New, rhs) => eval_expr(*rhs, old),
        Expr::Assign(_, _) => panic!("Only assignments where lhs equals new are supported!"),
        Expr::Old => old,
        Expr::New => panic!("Can't eval new!"),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}
