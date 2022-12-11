use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, text::Character};
use std::error::Error;

fn char_to_string(c: &char) -> String {
    if c.is_whitespace() {
        format!("{c:?}")
    } else {
        c.to_string()
    }
}

fn print_parser_error(input: &str, err: Simple<char>) {
    let msg = format!(
        "{}{}, expected {}",
        if err.found().is_some() {
            "Unexpected token"
        } else {
            "Unexpected end of input"
        },
        if let Some(label) = err.label() {
            format!(" while parsing {}", label)
        } else {
            String::new()
        },
        if err.expected().len() == 0 {
            "something else".to_string()
        } else {
            err.expected()
                .map(|expected| match expected {
                    // Some(expected) if expected.is_whitespace() => format!("{expected:?}"),
                    Some(expected) => char_to_string(expected),
                    None => "end of input".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        },
    );

    let report = Report::build(ReportKind::Error, (), err.span().start)
        .with_code(3)
        .with_message(msg)
        .with_label(
            Label::new(err.span())
                .with_message(format!(
                    "Unexpected {}",
                    err.found()
                        .map(|c| format!("token {}", char_to_string(c).fg(Color::Red)))
                        .unwrap_or_else(|| "end of input".to_string())
                ))
                .with_color(Color::Red),
        );

    let report = match err.reason() {
        chumsky::error::SimpleReason::Unclosed { span, delimiter } => report.with_label(
            Label::new(span.clone())
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_color(Color::Yellow),
        ),
        chumsky::error::SimpleReason::Unexpected => report,
        chumsky::error::SimpleReason::Custom(msg) => report.with_label(
            Label::new(err.span())
                .with_message(format!("{}", msg.fg(Color::Yellow)))
                .with_color(Color::Yellow),
        ),
    };

    report.finish().print(Source::from(&input)).unwrap();
}

// #[derive(Debug, Default)]
// struct Monkey {
//     starting_items: Vec<u32>,
//     operation_str: String,
//     divisible_by: u32,
//     target_if_true: usize,
//     target_if_false: usize,
// }

#[derive(Debug, Clone)]
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Old,
    New,
}

#[derive(Debug, Clone)]
enum MonkeyAction {
    ThrowToMonkey(u32),
}

#[derive(Debug, Clone)]
enum MonkeyBool {
    True,
    False,
}

#[derive(Debug, Clone)]
struct MonkeyTestCondition(MonkeyBool, MonkeyAction);

#[derive(Debug, Clone)]
enum MonkeyLang {
    MonkeyDefinition(u32, Vec<MonkeyLang>),
    StartingItems(Vec<i64>),
    Operation(Expr),
    Test {
        divisible_by: i64,
        conditions: Vec<MonkeyTestCondition>,
    },
    // TestCondition(MonkeyBool, MonkeyAction),
}

fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    // let int = text::int(10)
    //     .map(|s: String| Expr::Num(s.parse().unwrap()))
    //     .padded();
    let single_line_whitespace =
        filter::<char, _, Simple<char>>(|c: &char| c.is_whitespace() && *c != '\r' && *c != '\n')
            .repeated();

    let var_or_int = text::keyword("new")
        .to(Expr::New)
        .or(text::keyword("old")
            .to(Expr::Old)
            .or(text::int(10).map(|s: String| Expr::Num(s.parse().unwrap()))))
        .padded_by(single_line_whitespace);
    let op = |c| just(c).padded_by(single_line_whitespace);

    let product = var_or_int
        .clone()
        .then(
            op('*')
                .to(Expr::Mul as fn(_, _) -> _)
                .then(var_or_int)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    let sum = product
        .clone()
        .then(
            op('+')
                .to(Expr::Add as fn(_, _) -> _)
                .then(product)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    let assign = sum
        .clone()
        .then(
            op('=')
                .to(Expr::Assign as fn(_, _) -> _)
                .then(sum)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    assign
}

fn monkey_parser() -> impl Parser<char, Vec<MonkeyLang>, Error = Simple<char>> {
    let single_line_whitespace =
        filter::<char, _, Simple<char>>(|c: &char| c.is_whitespace() && *c != '\r' && *c != '\n')
            .repeated()
            .ignored();
    let int = text::int(10)
        .map(|s: String| s.parse::<i64>().unwrap())
        .padded_by(single_line_whitespace);
    let uint = text::int(10)
        .map(|s: String| s.parse::<u32>().unwrap())
        .padded_by(single_line_whitespace);
    let indention = |repeat| {
        filter::<char, _, Simple<char>>(|c: &char| c.is_whitespace() && *c != '\r' && *c != '\n')
            .repeated()
            .exactly(repeat)
            .ignored()
    };
    let newline = text::newline::<Simple<char>>().ignored();

    // let list_of_int = int
    //     // .clone()
    //     .then(
    //         just(',')
    //             .padded_by(single_line_whitespace)
    //             .then(int)
    //             .map(|(_, f)| f)
    //             .repeated(),
    //     )
    //     .map(|(first, rest)| {
    //         let mut list: Vec<i64> = Vec::new();
    //         list.push(first);
    //         list.extend(rest.iter());
    //         list
    //     });
    let list_of_int = int
        .clone()
        .padded_by(single_line_whitespace)
        .separated_by(just(','));

    let starting_items = just("Starting items:")
        .then(single_line_whitespace)
        .then(list_of_int)
        // .then(text::newline())
        .map(|((_, _), l)| MonkeyLang::StartingItems(l));

    let expr = expr_parser();
    let operation = just("Operation:")
        .then(single_line_whitespace)
        .then(expr)
        .map(|(_, e)| MonkeyLang::Operation(e));

    let true_or_false = text::keyword::<_, _, Simple<char>>("true")
        .to(MonkeyBool::True)
        .or(text::keyword("false").to(MonkeyBool::False));

    let action = just("throw to monkey")
        // .padded()
        .then(single_line_whitespace)
        .then(uint)
        .map(|(_, target)| MonkeyAction::ThrowToMonkey(target));

    let test_condition = text::keyword("If")
        .then(single_line_whitespace)
        .then(true_or_false)
        .then(just(':').ignored())
        .then(single_line_whitespace)
        // .padded()
        .then(action)
        .map(|((((_, cond), _), _), action)| MonkeyTestCondition(cond, action));

    let divisible_by = just("divisible by")
        // .padded()
        .then(single_line_whitespace)
        .then(int)
        .map(|(_, i)| i);

    let test = just("Test:")
        // .padded()
        .then(text::whitespace().ignored())
        .then(divisible_by)
        .then(newline.or_not())
        .then(
            // indention(4)
            text::whitespace()
                .then(test_condition)
                // .then(text::whitespace().repeated().ignored())
                // .then(newline)
                // .map(|(((_, cond), _), _)| cond)
                .map(|(_, cond)| cond)
                .repeated(),
        )
        .map(|(((_, divisible_by), _), conditions)| MonkeyLang::Test {
            divisible_by,
            conditions,
        });

    let monkey_def = just("Monkey")
        .padded()
        .then(uint)
        .then(just(':'))
        .then(newline)
        .then(
            (indention(2).then(starting_items).then(newline.or_not()))
                .or(indention(2).then(operation).then(newline.or_not()))
                .or(indention(2).then(test).then(newline.or_not()))
                .map(|((_, item), _)| {
                    println!("  $: {item:?}");
                    item
                })
                .repeated(),
        )
        // .map(|((((_, monkey), _), items), _)| {
        .map(|((((_, monkey), _), _), items)| {
            let def = MonkeyLang::MonkeyDefinition(monkey, items);
            println!("  $: {def:?}");
            def
        });

    // let monkey_def_block = monkey_def.then(
    //     starting_items
    //         .then(newline)
    //         .or(operation.then(newline))
    //         .or(test.then(newline))
    //         .repeated(),
    // );

    monkey_def.padded().repeated().then_ignore(end())
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

    match monkey_parser().parse(input.clone()) {
        Ok(tokens) => println!("\n{tokens:#?}"),
        Err(err) => err.into_iter().for_each(|e| {
            println!();
            // println!("{e:?}");
            print_parser_error(&input, e);
            println!();
        }),
    }

    Ok(())
}
