use std::iter::Peekable;

use couch_lang_lexer::{Token, TokenVariant};

#[derive(Debug, PartialEq)]
pub enum Expression {
    Int(i64),
    Float(f64),
    Call {
        subject: Box<Node<Expression>>,
        arguments: Vec<Node<Expression>>,
    },
    Identifier(String),
    Unary {
        subject: Box<Node<Expression>>,
        variant: UnaryVariant,
    },
    Binary {
        left: Box<Node<Expression>>,
        right: Box<Node<Expression>>,
        variant: BinaryVariant,
    },
}

#[derive(Debug, PartialEq)]
pub enum UnaryVariant {
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum BinaryVariant {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Debug, PartialEq)]
pub struct Node<T> {
    pub value: T,
    pub position: Position,
}

#[derive(Debug, PartialEq)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl From<&Token> for Position {
    fn from(token: &Token) -> Self {
        Self {
            index: token.index,
            column: token.column,
            line: token.line,
        }
    }
}

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    iter: Peekable<I>,
    text: String,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse_expression(&mut self) -> Node<Expression> {
        self.parse_add_subtract()
    }

    fn parse_add_subtract(&mut self) -> Node<Expression> {
        let left = self.parse_multiply_divide();
        let Some(operand) = self.iter.peek() else {
            return left;
        };
        if operand.variant == TokenVariant::Plus {
            let position = Position { ..left.position };
            self.iter.next().unwrap();
            let right = self.parse_add_subtract();
            return Self::node(
                Expression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    variant: BinaryVariant::Addition,
                },
                position,
            );
        } else if operand.variant == TokenVariant::Minus {
            let position = Position { ..left.position };
            self.iter.next().unwrap();
            let right = self.parse_add_subtract();
            return Self::node(
                Expression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    variant: BinaryVariant::Subtraction,
                },
                position,
            );
        } else {
            left
        }
    }
    fn parse_multiply_divide(&mut self) -> Node<Expression> {
        let left = self.parse_unary();
        let Some(operand) = self.iter.peek() else {
            return left;
        };
        if operand.variant == TokenVariant::Asterisk {
            let position = Position { ..left.position };
            self.iter.next().unwrap();
            let right = self.parse_multiply_divide();
            return Self::node(
                Expression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    variant: BinaryVariant::Multiplication,
                },
                position,
            );
        } else if operand.variant == TokenVariant::Slash {
            let position = Position { ..left.position };
            self.iter.next().unwrap();
            let right = self.parse_multiply_divide();
            return Self::node(
                Expression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    variant: BinaryVariant::Division,
                },
                position,
            );
        } else {
            left
        }
    }
    fn parse_unary(&mut self) -> Node<Expression> {
        let token = self.iter.peek().unwrap();
        match token.variant {
            TokenVariant::Minus => {
                let token = self.iter.next().unwrap();
                let subject = self.parse_unary();
                return Self::node(
                    Expression::Unary {
                        subject: Box::new(subject),
                        variant: UnaryVariant::Negate,
                    },
                    (&token).into(),
                );
            }
            _ => self.parse_operand(),
        }
    }
    fn parse_operand(&mut self) -> Node<Expression> {
        let token = self.iter.peek().unwrap();
        match token.variant {
            TokenVariant::Integer => {
                let token = self.iter.next().unwrap();
                let value = &self.text[token.index..token.index + token.length]
                    .parse::<i64>()
                    .unwrap();
                return Self::node(Expression::Int(*value), (&token).into());
            }
            TokenVariant::Float => {
                let token = self.iter.next().unwrap();
                let value = &self.text[token.index..token.index + token.length]
                    .parse::<f64>()
                    .unwrap();
                return Self::node(Expression::Float(*value), (&token).into());
            }
            _ => todo!(),
        }
    }
    fn node<T>(value: T, position: Position) -> Node<T> {
        Node { value, position }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use couch_lang_lexer::Lexer;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_integer() {
        let input = String::from("1");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_expression();
        assert_eq!(
            expression,
            Node {
                position: Position {
                    column: 1,
                    index: 0,
                    line: 1,
                },
                value: Expression::Int(1)
            }
        )
    }

    #[test]
    fn parse_negative_integer() {
        let input = String::from("-1");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_expression();
        assert_eq!(
            expression,
            Node {
                position: Position {
                    index: 0,
                    column: 1,
                    line: 1,
                },
                value: Expression::Unary {
                    subject: Box::new(Node {
                        value: Expression::Int(1),
                        position: Position {
                            index: 1,
                            line: 1,
                            column: 2,
                        },
                    }),
                    variant: UnaryVariant::Negate,
                },
            }
        )
    }

    #[test]
    fn parse_addition_subtraction() {
        let input = String::from("20 + 27 - 49.5");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_expression();
        assert_eq!(
            expression,
            Node {
                position: Position {
                    index: 0,
                    column: 1,
                    line: 1,
                },
                value: Expression::Binary {
                    left: Box::new(Node {
                        value: Expression::Int(20),
                        position: Position {
                            index: 0,
                            line: 1,
                            column: 1
                        },
                    }),
                    right: Box::new(Node {
                        position: Position {
                            index: 5,
                            column: 6,
                            line: 1,
                        },
                        value: Expression::Binary {
                            left: Box::new(Node {
                                value: Expression::Int(27),
                                position: Position {
                                    index: 5,
                                    column: 6,
                                    line: 1,
                                },
                            }),
                            right: Box::new(Node {
                                value: Expression::Float(49.5),
                                position: Position {
                                    index: 10,
                                    line: 1,
                                    column: 11,
                                },
                            }),
                            variant: BinaryVariant::Subtraction,
                        }
                    }),
                    variant: BinaryVariant::Addition,
                }
            }
        )
    }

    #[test]
    fn parse_multiplication_division() {
        let input = String::from("20 * 27 + 49.5");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_expression();
        assert_eq!(
            expression,
            Node {
                position: Position {
                    index: 0,
                    column: 1,
                    line: 1,
                },
                value: Expression::Binary {
                    right: Box::new(Node {
                        value: Expression::Float(49.5),
                        position: Position {
                            index: 10,
                            line: 1,
                            column: 11,
                        },
                    }),
                    left: Box::new(Node {
                        position: Position {
                            index: 0,
                            column: 1,
                            line: 1,
                        },
                        value: Expression::Binary {
                            left: Box::new(Node {
                                value: Expression::Int(20),
                                position: Position {
                                    index: 0,
                                    line: 1,
                                    column: 1
                                },
                            }),
                            right: Box::new(Node {
                                value: Expression::Int(27),
                                position: Position {
                                    index: 5,
                                    column: 6,
                                    line: 1,
                                },
                            }),
                            variant: BinaryVariant::Multiplication,
                        }
                    }),
                    variant: BinaryVariant::Addition,
                }
            }
        )
    }
}
