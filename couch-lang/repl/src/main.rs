use std::{
    env,
    io::{stdin, stdout, BufRead, Write},
};

use couch_lang_evaluator::Evaluator;
use couch_lang_lexer::{Lexer, Token};
use couch_lang_parser::Parser;

fn main() -> ! {
    loop {
        let mut stdin = stdin().lock();
        let mut buf = String::new();
        let args: Vec<String> = env::args().collect();
        print!("> ");
        stdout().flush().unwrap();
        stdin.read_line(&mut buf).unwrap();
        let lexer = Lexer::new(buf.chars());
        let tokens: Vec<Token> = lexer.into_iter().collect();
        if args.contains(&String::from("--tokens")) {
            println!("tokens -> {tokens:#?}");
        }
        let mut parser = Parser::new(tokens.into_iter(), buf.clone());
        let ast = parser.parse_statements();
        if args.contains(&String::from("--ast")) {
            println!("ast -> {ast:#?}");
        }
        let value = Evaluator::evaluate_statements(ast);
        println!("value -> {value:?}");
    }
}
