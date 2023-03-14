#![allow(dead_code)]

const NO_MUT_PEEK_NEXT_MESSAGE: &str = "should not mutate between peek & next";

const KEYWORDS: &[&str] = &["let", "mut"];

use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub struct Token {
    index: usize,
    length: usize,
    variant: TokenVariant,
    line: usize,
    column: usize,
}

#[derive(PartialEq, Debug)]
pub enum TokenVariant {
    Whitespace,
    Keyword,
    Identifier,
    LBrace,
    RBrace,
    Equal,
    PlusEqual,
    Plus,
    MinusEqual,
    Minus,
    AsteriskEqual,
    Asterisk,
    SlashEqual,
    Slash,
    Semicolon,
    Integer,
    Float,
    Error,
    String,
}

pub struct Lexer<I: Iterator<Item = char>> {
    iter: Peekable<I>,
    index: usize,
    line: usize,
    column: usize,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            column: 1,
            line: 1,
            index: 0,
        }
    }

    fn step(&mut self, new_line: bool, char_length: usize) {
        self.index += 1;
        if new_line {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += char_length;
        }
    }

    fn make_single_token(&mut self, variant: TokenVariant) -> Token {
        let value = self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
        let token = Token {
            index: self.index,
            length: value.len_utf8(),
            variant,
            line: self.line,
            column: self.column,
        };
        let is_newline = value == '\n';
        self.step(is_newline, value.len_utf8());
        token
    }

    fn make_keyword_or_identifier(&mut self) -> Token {
        let index = self.index;
        let column = self.column;
        let line = self.line;
        let mut text = String::new();

        let make_token = |text: String| {
            let variant = if KEYWORDS.contains(&text.as_str()) {
                TokenVariant::Keyword
            } else {
                TokenVariant::Identifier
            };
            Token {
                variant,
                index,
                length: text.len(),
                line,
                column,
            }
        };

        loop {
            let Some(char) = self.iter.peek() else {
                break make_token(text)
            };
            match char {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                    self.step(false, 1);
                    let char = self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                    text.push(char);
                }
                _ => break make_token(text),
            };
        }
    }

    fn make_number(&mut self) -> Token {
        let index = self.index;
        let column = self.column;
        let line = self.line;
        let mut length = 0;
        let mut dot_has_appeared = false;

        loop {
            let Some(char) = self.iter.peek() else {
                let variant = if dot_has_appeared {
                    TokenVariant::Float
                } else {
                    TokenVariant::Integer
                };
                return Token {
                    variant,
                    index,
                    length,
                    line,
                    column,
                };
            };
            match char {
                '0'..='9' => {
                    self.step(false, 1);
                    self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                    length += 1;
                }
                '.' if dot_has_appeared => {
                    self.step(false, 1);
                    self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                    length += 1;
                    break Token {
                        variant: TokenVariant::Error,
                        index,
                        length,
                        line,
                        column,
                    };
                }
                '.' => {
                    self.step(false, 1);
                    self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                    length += 1;
                    dot_has_appeared = true;
                }
                _ => {
                    let variant = if dot_has_appeared {
                        TokenVariant::Float
                    } else {
                        TokenVariant::Integer
                    };
                    break Token {
                        variant,
                        index,
                        length,
                        line,
                        column,
                    };
                }
            };
        }
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(char) = self.iter.peek() else {
            return None;
        };

        let token = match char {
            '0'..='9' => self.make_number(),
            ' ' => self.make_single_token(TokenVariant::Whitespace),
            '\n' => self.make_single_token(TokenVariant::Whitespace),
            '=' => self.make_single_token(TokenVariant::Equal),
            ';' => self.make_single_token(TokenVariant::Semicolon),
            'a'..='z' | 'A'..='Z' | '_' => self.make_keyword_or_identifier(),
            c => panic!("unrecognized character {c}"),
        };
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    struct TokenFactory {
        column: usize,
        line: usize,
        index: usize,
    }

    impl TokenFactory {
        pub fn new() -> Self {
            Self {
                column: 1,
                line: 1,
                index: 0,
            }
        }

        pub fn make(&mut self, text: &str, variant: TokenVariant) -> Token {
            let token = Token {
                variant,
                index: self.index,
                length: text.len(),
                line: self.line,
                column: self.column,
            };
            if text == "\n" {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += text.len();
            };
            self.index += text.len();
            token
        }
    }

    #[test]
    fn should_tokenize_numbers() {
        let input = String::from("500 500.0 5.");
        let lexer = Lexer::new(input.chars());
        let mut factory = TokenFactory::new();

        assert_eq!(
            lexer.collect::<Vec<Token>>(),
            vec![
                factory.make("500", TokenVariant::Integer),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("500.0", TokenVariant::Float),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("5.", TokenVariant::Float),
            ]
        );
    }

    #[test]
    fn should_have_assignments() {
        let input = String::from("let id_0 = 5;\nlet mut id_1 = 10;");

        let lexer = Lexer::new(input.chars());
        let mut factory = TokenFactory::new();

        assert_eq!(
            lexer.collect::<Vec<Token>>(),
            vec![
                factory.make("let", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("id_0", TokenVariant::Identifier),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("=", TokenVariant::Equal),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("5", TokenVariant::Integer),
                factory.make(";", TokenVariant::Semicolon),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make("let", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("mut", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("id_1", TokenVariant::Identifier),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("=", TokenVariant::Equal),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("10", TokenVariant::Integer),
                factory.make(";", TokenVariant::Semicolon),
            ]
        );
    }

    /*
    #[test]
    fn example_one() {
        let input = String::from(
            r#"// comment
    /* multiline
    comment */
    fn function_name() {
    let a = 5;
    let mut b = 3;
    b += a;
    return b;
}

let c = function_name(); // c += 1; // ERR: mutating a non-mutable variable
    let mut c = 8; // c += 0.5; // ERR: combining an integer with a non-float"#,
    );
    let _lexer = Lexer::new(input.chars());
}
*/
}
