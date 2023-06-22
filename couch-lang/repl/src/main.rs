use std::{
    env,
    io::{stdin, stdout, BufRead, Write},
};

use couch_lang_evaluator::Evaluator;
use couch_lang_lexer::{Lexer, Token};
use couch_lang_parser::Parser;

fn eval(code: String, print_tokens: bool, print_ast: bool) {
    let lexer = Lexer::new(code.chars());
    let tokens: Vec<Token> = lexer.into_iter().collect();
    if print_tokens {
        println!("tokens -> [");
        for token in tokens.iter() {
            println!(" {}", token.to_fancy_string(&code));
        }
        println!("]");
    }
    let mut parser = Parser::new(tokens.into_iter(), code.clone());
    let ast = parser.parse_statements();
    if print_ast {
        println!("ast -> [");
        for ast_item in ast.iter() {
            println!("{:#?}", ast_item);
        }
        println!("]");
    }
    let value = Evaluator::evaluate_statements(ast);
    println!("value -> {value:?}");
}

fn print_help() {
    println!("couch-lang-repl");
    println!("== flags: ==");
    println!("-h | --help   --> show this help text");
    println!("-t | --tokens --> include generated tokens with program output");
    println!("-a | --ast    --> include generated AST with program output");
    println!("-m | --multi  --> start REPL in multiline mode");
    println!("== commands ==");
    println!(":exit --> exit the program");
    println!(":eval --> (multiline) evaluate code in program buffer");
    println!(":show --> (multiline) show current program buffer");
    println!();
}

fn main() -> ! {
    let mut code_buffer = String::new();
    let print_tokens = env::args().find(|s| s == "--tokens" || s == "-t").is_some();
    let print_ast = env::args().find(|s| s == "--ast" || s == "-a").is_some();
    let multiline = env::args().find(|s| s == "--multi" || s == "-m").is_some();
    let help = env::args().find(|s| s == "--help" || s == "-h").is_some();
    if help {
        print_help();
    }
    loop {
        let mut stdin = stdin().lock();
        print!("> ");
        stdout().flush().unwrap();
        let mut line_buffer = String::new();
        stdin.read_line(&mut line_buffer).unwrap();
        if line_buffer.trim() == ":exit" {
            std::process::exit(0)
        } else if multiline && line_buffer.trim() == ":show" {
            print!("{code_buffer}");
        } else if multiline && line_buffer.trim() == ":eval" {
            eval(code_buffer.clone(), print_tokens, print_ast);
            code_buffer = "".to_owned();
        } else if !multiline {
            eval(line_buffer, print_tokens, print_ast);
        } else {
            code_buffer += &line_buffer;
        }
    }
}
