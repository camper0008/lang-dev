#![allow(dead_code)]

mod lexer;
mod parsed;
mod parser;
mod pos;
mod runtime;
mod token;
mod utils;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::pos::ErrorCollector;
use crate::runtime::Evaluator;

fn main() {
    //let text = "1 + 2 * -(3 - 4) + 1";
    let text = r#"{ 123; (123, "foobar"), [false, 123] }"#;

    println!("Text:\n{text}\n");

    let mut lexer_collector = ErrorCollector::new();
    let mut lexer = Lexer::new(text, &mut lexer_collector);

    let mut parser_collector = ErrorCollector::new();
    let mut parser = Parser::new(text, &mut lexer, &mut parser_collector);
    let parsed = parser.parse();

    let errors = parser_collector.merged_with(lexer_collector).errors();
    let failed = !errors.is_empty();
    if failed {
        println!("{} error(s) occurred:", errors.len());
        for error in errors {
            println!(
                "{}: {}, at {}:{}",
                error.error_type.to_string(),
                error.message,
                error.pos.line,
                error.pos.col,
            );
        }
        println!("");
    }

    println!("Parsed:\n{parsed:#?}\n");

    if failed {
        return;
    }

    let mut evaluator = Evaluator::new();
    let result = evaluator.eval_expr(&parsed);
    match result {
        Ok(value) => println!("Value:\n{value:?}"),
        Err(error) => println!(
            "{}: {}, at {}:{}",
            error.error_type.to_string(),
            error.message,
            error.pos.line,
            error.pos.col,
        ),
    };
}
