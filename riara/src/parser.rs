use crate::lexer::Lexer;
use crate::parsed::{BinaryType, Expr, UnaryType};
use crate::pos::{Error, ErrorCollector, ErrorType, Node, Pos};
use crate::token::{Token, TokenType};

pub struct Parser<'a> {
    text: &'a str,
    lexer: &'a mut Lexer<'a>,
    current: Token,
    error_collector: &'a mut ErrorCollector,
}

impl<'a> Parser<'a> {
    pub fn new(
        text: &'a str,
        lexer: &'a mut Lexer<'a>,
        error_collector: &'a mut ErrorCollector,
    ) -> Self {
        let current = lexer.next_token();
        Self {
            text,
            lexer,
            current,
            error_collector,
        }
    }

    pub fn parse(&mut self) -> Node<Expr> {
        let expr = self.parse_expr();
        match &self.current.token_type {
            TokenType::Eof => expr,
            tt => {
                let pos = self.current.pos.clone();
                self.add_error(pos, format!("expected end, got {tt:#?}"));
                expr
            }
        }
    }

    fn parse_expr(&mut self) -> Node<Expr> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Node<Expr> {
        let mut left = self.parse_factor();
        loop {
            match self.current.token_type {
                TokenType::Plus => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Add,
                            left: Box::new(left),
                            right: Box::new(self.parse_factor()),
                        },
                        pos,
                    )
                }
                TokenType::Minus => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Subtract,
                            left: Box::new(left),
                            right: Box::new(self.parse_factor()),
                        },
                        pos,
                    )
                }
                _ => break left,
            }
        }
    }

    fn parse_factor(&mut self) -> Node<Expr> {
        let mut left = self.parse_unary();
        loop {
            match self.current.token_type {
                TokenType::Asterisk => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Multiply,
                            left: Box::new(left),
                            right: Box::new(self.parse_unary()),
                        },
                        pos,
                    )
                }
                TokenType::Slash => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Divide,
                            left: Box::new(left),
                            right: Box::new(self.parse_unary()),
                        },
                        pos,
                    )
                }
                _ => break left,
            }
        }
    }

    fn parse_unary(&mut self) -> Node<Expr> {
        let pos = self.pos();
        match self.current.token_type {
            TokenType::Plus => {
                self.step();
                Node::new(
                    Expr::Unary {
                        unary_type: UnaryType::Plus,
                        subject: Box::new(self.parse_unary()),
                    },
                    pos,
                )
            }
            TokenType::Minus => {
                self.step();
                Node::new(
                    Expr::Unary {
                        unary_type: UnaryType::Negate,
                        subject: Box::new(self.parse_unary()),
                    },
                    pos,
                )
            }
            _ => self.parse_operand(),
        }
    }

    fn parse_operand(&mut self) -> Node<Expr> {
        match &self.current.token_type {
            TokenType::Int => {
                let current = self.current.clone();
                let number = current.value(self.text).parse().unwrap();
                self.step();
                Node::new(Expr::Int(number), current.pos)
            }
            TokenType::String => {
                let current = self.current.clone();
                let str = String::from(current.value(self.text));
                self.step();
                Node::new(Expr::String(str), current.pos)
            }
            TokenType::LParen => {
                self.step();
                let left = self.parse_expr();
                match self.current.token_type {
                    TokenType::RParen => self.step(),
                    _ => self.add_error(
                        self.pos(),
                        "you forgot a closing parenthesis idiot".to_string(),
                    ),
                };
                left
            }
            token_type => {
                self.add_error(self.pos(), format!("expected operand, got {token_type:#?}"));
                Node::new(Expr::Error, self.pos())
            }
        }
    }

    fn pos(&self) -> Pos {
        self.current.pos.clone()
    }

    fn step(&mut self) {
        self.current = self.lexer.next_token();
    }

    fn add_error(&mut self, pos: Pos, message: String) {
        self.error_collector.add(Error {
            error_type: ErrorType::Parser,
            pos,
            message,
        })
    }
}
