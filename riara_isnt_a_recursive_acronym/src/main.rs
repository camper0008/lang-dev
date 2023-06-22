
mod pos;
mod token;
mod lexer;
mod parsed;
mod parser;

#![allow(dead_code)]

use create::pos::{ErrorType, ErrorCollector};
use create::lexer::{Lexer};
use create::parser::{Parser};

fn main() {
    let text = "1 + 2 * -(3 - 4) + 1";

    println!("Text: \"{text}\"");

    let mut lexer_collector = ErrorCollector::new();
    let mut lexer = Lexer::new(text, &mut lexer_collector);

    let mut parser_collector = ErrorCollector::new();
    let mut parser = Parser::new(text, &mut lexer, &mut parser_collector);
    let parsed = parser.parse();

    let errors = parser_collector.merged_with(lexer_collector).errors();
    if !errors.is_empty() {
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
    }

    println!("Parsed: {expr:#?}");

    let result = eval_expr(expr);
    println!("Value: {result:?}");
}
