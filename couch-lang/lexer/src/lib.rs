pub mod indexed_char_iterator;

const NO_MUT_PEEK_NEXT_MESSAGE: &str = "should not mutate between peek & next";

use std::iter::Peekable;

use indexed_char_iterator::{IndexedChar, IndexedCharIterator};
use token_variant::TokenVariant;

mod token_variant;

pub struct Lexer<I: Iterator<Item = char>> {
    iter: Peekable<IndexedCharIterator<I>>,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub index: usize,
    pub length: usize,
    pub variant: TokenVariant,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn to_fancy_string(&self, input: &str) -> String {
        format!(
            "[{}:{}]\t{}({})",
            self.column,
            self.line,
            self.variant,
            &input[self.index..self.index + self.length]
        )
    }
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

        Token {
            length: value.len_utf8(),
            index,
            variant,
            line,
            column,
        }
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
            let variant = match text.as_str() {
                "fn" => TokenVariant::FnKeyword,
                "let" => TokenVariant::LetKeyword,
                "mut" => TokenVariant::MutKeyword,
                "return" => TokenVariant::ReturnKeyword,
                _ => TokenVariant::Identifier,
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
        let single_token = self.make_single_token(TokenVariant::Slash);
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
                    break self.make_token();
                }
                self.iter.next();
            },
            Some(IndexedChar { value: '=', .. }) => {
                self.iter.next();
                Some(Token {
                    index: single_token.index,
                    column: single_token.column,
                    line: single_token.line,
                    length: 2,
                    variant: TokenVariant::SlashEqual,
                })
            }
            Some(_) | None => Some(single_token),
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
                length: 1,
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
                    index,
                    length,
                    variant,
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
                        index,
                        length,
                        variant,
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
            ' ' | '\n' => {
                self.iter.next();
                self.make_token()?
            }
            '=' => self.make_single_or_double_token(
                TokenVariant::Equal,
                '=',
                TokenVariant::DoubleEqual,
            ),
            ';' => self.make_single_token(TokenVariant::Semicolon),
            '(' => self.make_single_token(TokenVariant::LParenthesis),
            ')' => self.make_single_token(TokenVariant::RParenthesis),
            '{' => self.make_single_token(TokenVariant::LBrace),
            '}' => self.make_single_token(TokenVariant::RBrace),
            '-' => {
                self.make_single_or_double_token(TokenVariant::Minus, '=', TokenVariant::MinusEqual)
            }
            '!' => self.make_single_or_double_token(
                TokenVariant::Exclamation,
                '=',
                TokenVariant::ExclamationEqual,
            ),
            '*' => self.make_single_or_double_token(
                TokenVariant::Asterisk,
                '=',
                TokenVariant::AsteriskEqual,
            ),
            '+' => {
                self.make_single_or_double_token(TokenVariant::Plus, '=', TokenVariant::PlusEqual)
            }
            'a'..='z' | 'A'..='Z' | '_' => self.make_keyword_or_identifier(),
            '/' => self.make_comment_or_slash()?,
            _ => {
                let token = Token {
                    variant: TokenVariant::Error,
                    index: char.index,
                    length: char.value.len_utf8(),
                    line: char.line,
                    column: char.column,
                };
                self.iter.next().expect(NO_MUT_PEEK_NEXT_MESSAGE);
                token
            }
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

        pub fn skip(&mut self, text: &str) -> Option<Token> {
            for c in text.chars() {
                self.make(&c.to_string(), TokenVariant::Error);
            }
            None
        }

        pub fn make(&mut self, text: &str, variant: TokenVariant) -> Option<Token> {
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
            Some(token)
        }
    }

    #[test]
    fn should_tokenize_numbers() {
        let input = String::from("500 500.0 5.");
        let lexer = Lexer::new(input.chars());
        let mut factory = TokenFactory::new();

        let tokens: Vec<Token> = vec![
            factory.make("500", TokenVariant::Integer),
            factory.skip(" "),
            factory.make("500.0", TokenVariant::Float),
            factory.skip(" "),
            factory.make("5.", TokenVariant::Float),
        ]
        .into_iter()
        .filter_map(|option| option)
        .collect();

        assert_eq!(lexer.into_iter().collect::<Vec<Token>>(), tokens);
    }

    #[test]
    fn should_have_assignments() {
        let input = String::from("let id_0 = 5;\nlet mut id_1 = 10;");

        let lexer = Lexer::new(input.chars());
        let mut factory = TokenFactory::new();

        let tokens: Vec<Token> = vec![
            factory.make("let", TokenVariant::LetKeyword),
            factory.skip(" "),
            factory.make("id_0", TokenVariant::Identifier),
            factory.skip(" "),
            factory.make("=", TokenVariant::Equal),
            factory.skip(" "),
            factory.make("5", TokenVariant::Integer),
            factory.make(";", TokenVariant::Semicolon),
            factory.skip("\n"),
            factory.make("let", TokenVariant::LetKeyword),
            factory.skip(" "),
            factory.make("mut", TokenVariant::MutKeyword),
            factory.skip(" "),
            factory.make("id_1", TokenVariant::Identifier),
            factory.skip(" "),
            factory.make("=", TokenVariant::Equal),
            factory.skip(" "),
            factory.make("10", TokenVariant::Integer),
            factory.make(";", TokenVariant::Semicolon),
        ]
        .into_iter()
        .filter_map(|option| option)
        .collect();

        assert_eq!(lexer.into_iter().collect::<Vec<Token>>(), tokens);
    }

    #[test]
    fn all_tokens() {
        let input =
            String::from("let mut fn return a ( ) { } = += + -= - *= * /= / ; 100 100.0 ! != == Å");

        let lexer = Lexer::new(input.chars());
        let mut factory = TokenFactory::new();

        use TokenVariant::*;

        let tokens: Vec<Token> = vec![
            factory.make("let", LetKeyword),
            factory.skip(" "),
            factory.make("mut", MutKeyword),
            factory.skip(" "),
            factory.make("fn", FnKeyword),
            factory.skip(" "),
            factory.make("return", ReturnKeyword),
            factory.skip(" "),
            factory.make("a", Identifier),
            factory.skip(" "),
            factory.make("(", LParenthesis),
            factory.skip(" "),
            factory.make(")", RParenthesis),
            factory.skip(" "),
            factory.make("{", LBrace),
            factory.skip(" "),
            factory.make("}", RBrace),
            factory.skip(" "),
            factory.make("=", Equal),
            factory.skip(" "),
            factory.make("+=", PlusEqual),
            factory.skip(" "),
            factory.make("+", Plus),
            factory.skip(" "),
            factory.make("-=", MinusEqual),
            factory.skip(" "),
            factory.make("-", Minus),
            factory.skip(" "),
            factory.make("*=", AsteriskEqual),
            factory.skip(" "),
            factory.make("*", Asterisk),
            factory.skip(" "),
            factory.make("/=", SlashEqual),
            factory.skip(" "),
            factory.make("/", Slash),
            factory.skip(" "),
            factory.make(";", Semicolon),
            factory.skip(" "),
            factory.make("100", Integer),
            factory.skip(" "),
            factory.make("100.0", Float),
            factory.skip(" "),
            factory.make("!", Exclamation),
            factory.skip(" "),
            factory.make("!=", ExclamationEqual),
            factory.skip(" "),
            factory.make("==", DoubleEqual),
            factory.skip(" "),
            factory.make("Å", Error),
        ]
        .into_iter()
        .filter_map(|option| option)
        .collect();

        assert_eq!(lexer.into_iter().collect::<Vec<Token>>(), tokens);
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

        let tokens: Vec<Token> = vec![
            factory.skip("// comment\n/* multiline\ncomment */\n"),
            factory.make("fn", TokenVariant::FnKeyword),
            factory.skip(" "),
            factory.make("function_name", TokenVariant::Identifier),
            factory.make("(", TokenVariant::LParenthesis),
            factory.make(")", TokenVariant::RParenthesis),
            factory.skip(" "),
            factory.make("{", TokenVariant::LBrace),
            factory.skip("\n    "),
            factory.make("let", TokenVariant::LetKeyword),
            factory.skip(" "),
            factory.make("mut", TokenVariant::MutKeyword),
            factory.skip(" "),
            factory.make("a", TokenVariant::Identifier),
            factory.skip(" "),
            factory.make("=", TokenVariant::Equal),
            factory.skip(" "),
            factory.make("5", TokenVariant::Integer),
            factory.make(";", TokenVariant::Semicolon),
            factory.skip("\n    "),
            factory.make("a", TokenVariant::Identifier),
            factory.skip(" "),
            factory.make("+=", TokenVariant::PlusEqual),
            factory.skip(" "),
            factory.make("5", TokenVariant::Integer),
            factory.make(";", TokenVariant::Semicolon),
            factory.skip("\n    "),
            factory.make("return", TokenVariant::ReturnKeyword),
            factory.skip(" "),
            factory.make("a", TokenVariant::Identifier),
            factory.make(";", TokenVariant::Semicolon),
            factory.skip("\n"),
            factory.make("}", TokenVariant::RBrace),
            factory.skip("\n\n"),
            factory.make("let", TokenVariant::LetKeyword),
            factory.skip(" "),
            factory.make("b", TokenVariant::Identifier),
            factory.skip(" "),
            factory.make("=", TokenVariant::Equal),
            factory.skip(" "),
            factory.make("function_name", TokenVariant::Identifier),
            factory.make("(", TokenVariant::LParenthesis),
            factory.make(")", TokenVariant::RParenthesis),
            factory.make(";", TokenVariant::Semicolon),
        ]
        .into_iter()
        .filter_map(|option| option)
        .collect();

        assert_eq!(lexer.into_iter().collect::<Vec<Token>>(), tokens);
    }
}
