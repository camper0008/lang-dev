use std::iter::Peekable;

use couch_lang_lexer::{Token, TokenVariant};

#[derive(Debug, PartialEq)]
pub enum Statement {
    Let {
        mutable: bool,
        identifier: Box<Node<Expression>>,
        value: Box<Node<Expression>>,
    },
    Return(Option<Box<Node<Expression>>>),
}

pub struct Parameter {
    mutable: bool,
    identifier: Box<Node<Expression>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Integer(i64),
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
    Assignment {
        left: Box<Node<Expression>>,
        right: Box<Node<Expression>>,
        variant: AssignmentVariant,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryVariant {
    NegateNumber,
    NegateBool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryVariant {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentVariant {
    Base,
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node<T> {
    pub value: T,
    pub position: Position,
}

#[derive(Debug, PartialEq, Clone)]
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
    pub fn new(iter: I, text: String) -> Self {
        Self {
            iter: iter.peekable(),
            text,
        }
    }
    pub fn parse_statement(&mut self) -> Node<Statement> {
        let Some(keyword) = self.iter.peek() else {
            todo!("EOF handling");
        };
        match &keyword.variant {
            TokenVariant::ReturnKeyword => self.parse_return(),
            TokenVariant::LetKeyword => self.parse_let(),
            variant => todo!("unexpected variant {variant:#?}"),
        }
    }
    pub fn parse_return(&mut self) -> Node<Statement> {
        let keyword = self
            .iter
            .next()
            .expect("parse return got called out of order");
        debug_assert_eq!(
            keyword.variant,
            TokenVariant::ReturnKeyword,
            "parse return got called out of order"
        );
        let position: Position = (&keyword).into();
        let token = self.iter.peek().expect("missing semicolon"); // TODO: pretty errors, not
                                                                  // expect
        match token.variant {
            TokenVariant::Semicolon => Self::node(Statement::Return(None), position),
            _ => Self::node(
                Statement::Return(Some(Box::new(self.parse_expression()))),
                position,
            ),
        }
    }
    pub fn parse_let(&mut self) -> Node<Statement> {
        let keyword = self.iter.peek().expect("called out of order");
        debug_assert_eq!(
            keyword.variant,
            TokenVariant::LetKeyword,
            "called out of order"
        );

        let position = Position {
            index: keyword.index,
            line: keyword.line,
            column: keyword.column,
        };

        self.iter.next();

        let Parameter {
            mutable,
            identifier,
        } = self.parse_parameter().value;

        // TODO: pretty error, not panic
        let next = self.iter.peek().expect("did not expect EOF");

        if !matches!(next.variant, TokenVariant::Equal) {
            // TODO: pretty error, not panic
            panic!("expected 'Equal', got {:#?}", next.variant);
        }

        self.iter.next();

        self.iter.peek().expect("did not expect EOF"); // TODO: pretty errors, not
                                                       // expect
                                                       // expression

        let value = self.parse_expression();

        // TODO: pretty error, not panic
        let next = self.iter.peek().expect("did not expect EOF");

        if !matches!(next.variant, TokenVariant::Semicolon) {
            // TODO: pretty error, not panic
            panic!("expected 'Semicolon', got {:#?}", next.variant);
        }

        self.iter.next();

        Self::node(
            Statement::Let {
                mutable,
                identifier,
                value: Box::new(value),
            },
            position,
        )
    }
    pub fn parse_parameter(&mut self) -> Node<Parameter> {
        // TODO: pretty error, not expect
        let next = self.iter.peek().expect("did not expect EOF");
        let position = Position {
            index: next.index,
            line: next.line,
            column: next.column,
        };
        let mutable = next.variant == TokenVariant::MutKeyword;
        if mutable {
            self.iter.next();
        }
        let identifier = self.parse_operand();
        if !matches!(identifier.value, Expression::Identifier(_)) {
            // TODO: pretty errors, not panic
            panic!("expected identifier, got {:#?}", identifier.value);
        }
        Self::node(
            Parameter {
                mutable,
                identifier: Box::new(identifier),
            },
            position,
        )
    }
    pub fn parse_assignment(&mut self) -> Node<Expression> {
        let left = self.parse_expression();
        let Some(operand) = self.iter.peek() else {
            return left;
        };

        let variant = match operand.variant {
            TokenVariant::Equal => AssignmentVariant::Base,
            TokenVariant::AsteriskEqual => AssignmentVariant::Multiplication,
            TokenVariant::MinusEqual => AssignmentVariant::Subtraction,
            TokenVariant::PlusEqual => AssignmentVariant::Addition,
            TokenVariant::SlashEqual => AssignmentVariant::Division,
            _ => return left,
        };
        let position = Position { ..left.position };
        self.iter.next().unwrap();
        let right = self.parse_equality();
        Self::node(
            Expression::Assignment {
                left: Box::new(left),
                right: Box::new(right),
                variant,
            },
            position,
        )
    }

    pub fn parse_expression(&mut self) -> Node<Expression> {
        self.parse_equality()
    }

    pub fn parse_equality(&mut self) -> Node<Expression> {
        let left = self.parse_add_subtract();
        let Some(operand) = self.iter.peek() else {
            return left;
        };
        let variant = match operand.variant {
            TokenVariant::DoubleEqual => BinaryVariant::Equal,
            TokenVariant::ExclamationEqual => BinaryVariant::NotEqual,
            _ => return left,
        };
        let position = Position { ..left.position };
        self.iter.next().unwrap();
        let right = self.parse_equality();
        Self::node(
            Expression::Binary {
                left: Box::new(left),
                right: Box::new(right),
                variant,
            },
            position,
        )
    }

    fn parse_add_subtract(&mut self) -> Node<Expression> {
        let left = self.parse_multiply_divide();
        let Some(operand) = self.iter.peek() else {
            return left;
        };
        let variant = match operand.variant {
            TokenVariant::Plus => BinaryVariant::Addition,
            TokenVariant::Minus => BinaryVariant::Subtraction,
            _ => return left,
        };
        let position = Position { ..left.position };
        self.iter.next().unwrap();
        let right = self.parse_add_subtract();
        Self::node(
            Expression::Binary {
                left: Box::new(left),
                right: Box::new(right),
                variant,
            },
            position,
        )
    }
    fn parse_multiply_divide(&mut self) -> Node<Expression> {
        let left = self.parse_unary();
        let Some(operand) = self.iter.peek() else {
            return left;
        };
        let variant = match operand.variant {
            TokenVariant::Asterisk => BinaryVariant::Multiplication,
            TokenVariant::Slash => BinaryVariant::Division,
            _ => return left,
        };
        let position = Position { ..left.position };
        self.iter.next().unwrap();
        let right = self.parse_multiply_divide();
        Self::node(
            Expression::Binary {
                left: Box::new(left),
                right: Box::new(right),
                variant,
            },
            position,
        )
    }
    fn parse_unary(&mut self) -> Node<Expression> {
        let token = self.iter.peek().expect("unexpected EOF");
        let variant = match token.variant {
            TokenVariant::Minus => UnaryVariant::NegateNumber,
            TokenVariant::Exclamation => UnaryVariant::NegateBool,
            _ => return self.parse_operand(), // TODO: member index call;
        };
        let token = self.iter.next().unwrap();
        let subject = self.parse_unary();
        Self::node(
            Expression::Unary {
                subject: Box::new(subject),
                variant,
            },
            (&token).into(),
        )
    }
    fn parse_operand(&mut self) -> Node<Expression> {
        let token = self.iter.peek().unwrap();
        match token.variant {
            TokenVariant::Identifier => {
                let token = self.iter.next().unwrap();
                let value = self.text[token.index..token.index + token.length].to_owned();
                Self::node(Expression::Identifier(value), (&token).into())
            }
            TokenVariant::Integer => {
                let token = self.iter.next().unwrap();
                let value = &self.text[token.index..token.index + token.length]
                    .parse::<i64>()
                    .unwrap();
                Self::node(Expression::Integer(*value), (&token).into())
            }
            TokenVariant::Float => {
                let token = self.iter.next().unwrap();
                let value = &self.text[token.index..token.index + token.length]
                    .parse::<f64>()
                    .unwrap();
                Self::node(Expression::Float(*value), (&token).into())
            }
            _ => todo!("unrecognized operand"),
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
                value: Expression::Integer(1)
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
                        value: Expression::Integer(1),
                        position: Position {
                            index: 1,
                            line: 1,
                            column: 2,
                        },
                    }),
                    variant: UnaryVariant::NegateNumber,
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
                        value: Expression::Integer(20),
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
                                value: Expression::Integer(27),
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
                                value: Expression::Integer(20),
                                position: Position {
                                    index: 0,
                                    line: 1,
                                    column: 1
                                },
                            }),
                            right: Box::new(Node {
                                value: Expression::Integer(27),
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

    #[test]
    fn parse_return() {
        let input = String::from("return;");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_statement();
        assert_eq!(
            expression,
            Node {
                position: Position {
                    index: 0,
                    column: 1,
                    line: 1,
                },
                value: Statement::Return(None),
            }
        )
    }

    #[test]
    fn parse_return_expression() {
        let input = String::from("return a + b;");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_statement();
        assert_eq!(
            expression,
            Node {
                position: Position {
                    index: 0,
                    column: 1,
                    line: 1,
                },
                value: Statement::Return(Some(Box::new(Node {
                    value: Expression::Binary {
                        left: Box::new(Node {
                            value: Expression::Identifier("a".to_string()),
                            position: Position {
                                index: 7,
                                line: 1,
                                column: 8,
                            },
                        }),
                        right: Box::new(Node {
                            value: Expression::Identifier("b".to_string()),
                            position: Position {
                                index: 11,
                                line: 1,
                                column: 12,
                            },
                        }),
                        variant: BinaryVariant::Addition,
                    },
                    position: Position {
                        index: 7,
                        line: 1,
                        column: 8,
                    },
                })))
            }
        )
    }

    #[test]
    fn parse_let_expression() {
        let input = String::from("let a = b;");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_statement();
        assert_eq!(
            expression,
            Node {
                value: Statement::Let {
                    mutable: false,
                    identifier: Box::new(Node {
                        value: Expression::Identifier("a".to_string()),
                        position: Position {
                            index: 4,
                            line: 1,
                            column: 5,
                        },
                    }),
                    value: Box::new(Node {
                        value: Expression::Identifier("b".to_string()),
                        position: Position {
                            index: 8,
                            line: 1,
                            column: 9,
                        },
                    }),
                },
                position: Position {
                    index: 0,
                    line: 1,
                    column: 1
                }
            }
        )
    }

    #[test]
    fn parse_let_mut_expression() {
        let input = String::from("let mut a = b;");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser {
            iter: lexer.into_iter().peekable(),
            text: input.clone(),
        };
        let expression = parser.parse_statement();
        assert_eq!(
            expression,
            Node {
                value: Statement::Let {
                    mutable: true,
                    identifier: Box::new(Node {
                        value: Expression::Identifier("a".to_string()),
                        position: Position {
                            index: 8,
                            line: 1,
                            column: 9,
                        },
                    }),
                    value: Box::new(Node {
                        value: Expression::Identifier("b".to_string()),
                        position: Position {
                            index: 12,
                            line: 1,
                            column: 13,
                        },
                    }),
                },
                position: Position {
                    index: 0,
                    line: 1,
                    column: 1
                }
            }
        )
    }

    #[test]
    fn add_float_and_int() {
        let input = String::from("2.5 + 4");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();
        assert_eq!(
            expression,
            Node {
                value: Expression::Binary {
                    left: Box::new(Node {
                        value: Expression::Float(2.5,),
                        position: Position {
                            index: 0,
                            line: 1,
                            column: 1,
                        },
                    }),
                    right: Box::new(Node {
                        value: Expression::Integer(4,),
                        position: Position {
                            index: 6,
                            line: 1,
                            column: 7,
                        },
                    }),
                    variant: BinaryVariant::Addition,
                },
                position: Position {
                    index: 0,
                    line: 1,
                    column: 1
                }
            }
        );
    }
}
