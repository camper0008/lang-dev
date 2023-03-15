#![allow(dead_code)]

pub mod indexed_char_iterator;

const NO_MUT_PEEK_NEXT_MESSAGE: &str = "should not mutate between peek & next";

const KEYWORDS: &[&str] = &["let", "mut", "fn", "return"];

use std::iter::Peekable;

use indexed_char_iterator::{IndexedChar, IndexedCharIterator};

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
    LParenthesis,
    RParenthesis,
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
    iter: Peekable<IndexedCharIterator<I>>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: IndexedCharIterator::new(iter).peekable(),
        }
    }

    fn make_single_token(&mut self, variant: TokenVariant) -> Token {
        let IndexedChar {
            line,
            column,
            index,
            value,
        } = self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);

        let token = Token {
            length: value.len_utf8(),
            index,
            variant,
            line,
            column,
        };
        token
    }

    fn make_keyword_or_identifier(&mut self) -> Token {
        let IndexedChar {
            index,
            line,
            column,
            value,
        } = self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
        let mut text = String::from(value);

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
            let Some(IndexedChar { value, .. }) = self.iter.peek() else {
                break make_token(text)
            };
            match value {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                    let IndexedChar { value, .. } =
                        self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                    text.push(value);
                }
                _ => break make_token(text),
            };
        }
    }

    fn make_comment_or_slash(&mut self) -> Option<Token> {
        self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
        match self.iter.peek() {
            Some(IndexedChar { value: '*', .. }) => {
                self.iter.next();
                loop {
                    let next = self.iter.peek()?;
                    if next.value == '*' {
                        self.iter.next();
                        let next = self.iter.peek()?;
                        if next.value == '/' {
                            self.iter.next();
                            break self.make_token();
                        }
                    } else {
                        self.iter.next();
                    }
                }
            }
            Some(IndexedChar { value: '/', .. }) => loop {
                let next = self.iter.peek()?;
                if next.value == '\n' {
                    break Some(self.make_single_token(TokenVariant::Whitespace));
                } else {
                    self.iter.next();
                }
            },
            Some(_) | None => Some(self.make_single_token(TokenVariant::Slash)),
        }
    }

    fn make_single_or_double_token(
        &mut self,
        single_variant: TokenVariant,
        second_char: char,
        second_variant: TokenVariant,
    ) -> Token {
        let IndexedChar {
            index,
            line,
            column,
            ..
        } = self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);

        match self.iter.peek() {
            Some(IndexedChar { value, .. }) if *value == second_char => {
                self.iter.next();
                Token {
                    index,
                    column,
                    line,
                    length: 2,
                    variant: second_variant,
                }
            }
            Some(_) | None => Token {
                index,
                column,
                line,
                length: 2,
                variant: single_variant,
            },
        }
    }

    fn make_number(&mut self) -> Token {
        let IndexedChar {
            index,
            line,
            column,
            ..
        } = self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
        let mut length = 1;
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
            match char.value {
                '0'..='9' => {
                    self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                    length += 1;
                }
                '.' if dot_has_appeared => {
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

    fn make_token(&mut self) -> Option<Token> {
        let Some(char) = self.iter.peek() else {
            return None;
        };

        let token = match char.value {
            '0'..='9' => self.make_number(),
            ' ' => self.make_single_token(TokenVariant::Whitespace),
            '\n' => self.make_single_token(TokenVariant::Whitespace),
            '=' => self.make_single_token(TokenVariant::Equal),
            ';' => self.make_single_token(TokenVariant::Semicolon),
            '(' => self.make_single_token(TokenVariant::LParenthesis),
            ')' => self.make_single_token(TokenVariant::RParenthesis),
            '{' => self.make_single_token(TokenVariant::LBrace),
            '}' => self.make_single_token(TokenVariant::RBrace),
            '+' => {
                self.make_single_or_double_token(TokenVariant::Plus, '=', TokenVariant::PlusEqual)
            }
            'a'..='z' | 'A'..='Z' | '_' => self.make_keyword_or_identifier(),
            '/' => match self.make_comment_or_slash() {
                Some(token) => token,
                None => return None,
            },
            c => panic!("unrecognized character {c}"),
        };
        Some(token)
    }
}

pub struct LexerIterator<I>
where
    I: Iterator<Item = char>,
{
    lexer: Lexer<I>,
}

impl<I> Iterator for LexerIterator<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.make_token()
    }
}

impl<I> IntoIterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;
    type IntoIter = LexerIterator<I>;

    fn into_iter(self) -> Self::IntoIter {
        LexerIterator { lexer: self }
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

        pub fn skip(&mut self, text: &str) {
            for c in text.chars() {
                self.make(&c.to_string(), TokenVariant::Whitespace);
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
            lexer.into_iter().collect::<Vec<Token>>(),
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
            lexer.into_iter().collect::<Vec<Token>>(),
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

    #[test]
    fn example_one() {
        let input = String::from(
            r#"// comment
/* multiline
comment */
fn function_name() {
    let mut a = 3;
    a += 5;
    return a;
}

let b = function_name();"#,
        );

        let lexer = Lexer::new(input.chars());
        let mut factory = TokenFactory::new();

        factory.skip("// comment");
        let newline_token = factory.make("\n", TokenVariant::Whitespace);
        factory.skip("/* multiline\ncomment */");

        assert_eq!(
            lexer.into_iter().collect::<Vec<Token>>(),
            vec![
                newline_token,
                factory.make("\n", TokenVariant::Whitespace),
                factory.make("fn", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("function_name", TokenVariant::Identifier),
                factory.make("(", TokenVariant::LParenthesis),
                factory.make(")", TokenVariant::RParenthesis),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("{", TokenVariant::LBrace),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("let", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("mut", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("a", TokenVariant::Identifier),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("=", TokenVariant::Equal),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("5", TokenVariant::Integer),
                factory.make(";", TokenVariant::Semicolon),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("a", TokenVariant::Identifier),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("+=", TokenVariant::PlusEqual),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("5", TokenVariant::Integer),
                factory.make(";", TokenVariant::Semicolon),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("return", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("a", TokenVariant::Identifier),
                factory.make(";", TokenVariant::Semicolon),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make("}", TokenVariant::RBrace),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make("\n", TokenVariant::Whitespace),
                factory.make("let", TokenVariant::Keyword),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("b", TokenVariant::Identifier),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("=", TokenVariant::Equal),
                factory.make(" ", TokenVariant::Whitespace),
                factory.make("function_name", TokenVariant::Identifier),
                factory.make("(", TokenVariant::LParenthesis),
                factory.make(")", TokenVariant::RParenthesis),
                factory.make(";", TokenVariant::Semicolon),
            ]
        );
    }
}
