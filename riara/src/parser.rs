use crate::lexer::Lexer;
use crate::parsed::{BinaryType, Expr, UnaryType};
use crate::pos::{Error, ErrorCollector, ErrorType, Node, Pos};
use crate::token::{Token, TokenType};
use crate::utils::unescape_string;

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
                self.add_error(pos, format!("expected Eof, got {tt:#?}"));
                expr
            }
        }
    }

    fn parse_statement(&mut self) -> Node<Expr> {
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> Node<Expr> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Node<Expr> {
        let mut left = self.parse_factor();
        loop {
            match self.current.token_type {
                TokenType::Plus => {
                    let pos = self.current.pos.clone();
                    self.step();
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
                    let pos = self.current.pos.clone();
                    self.step();
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
                    let pos = self.current.pos.clone();
                    self.step();
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
                    let pos = self.current.pos.clone();
                    self.step();
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
        let pos = self.current.pos.clone();
        match &self.current.token_type {
            TokenType::Int => {
                let number = self.current.value(self.text).parse().unwrap();
                self.step();
                Node::new(Expr::Int(number), pos)
            }
            TokenType::String => self.parse_string_expr(),
            TokenType::False => {
                self.step();
                Node::new(Expr::Bool(false), pos)
            }
            TokenType::True => {
                self.step();
                Node::new(Expr::Bool(true), pos)
            }
            TokenType::LParen => self.parse_unit_or_group_or_tuple_expr(),
            TokenType::LBrace => self.parse_block_expr(),
            TokenType::LBracket => self.parse_array_expr(),
            token_type => {
                self.add_error(self.pos(), format!("expected operand, got {token_type:#?}"));
                Node::new(Expr::Error, self.pos())
            }
        }
    }

    fn parse_string_expr(&mut self) -> Node<Expr> {
        let pos = self.current.pos.clone();
        let literal_value = self.current.value(self.text);
        let unescaped_value = match unescape_string(&literal_value[1..literal_value.len() - 1]) {
            Ok(value) => value,
            Err(message) => {
                self.add_error(pos.clone(), format!("malformed string, {message}"));
                return Node::new(Expr::Error, pos);
            }
        };
        self.step();
        Node::new(Expr::String(unescaped_value), pos)
    }

    fn parse_unit_or_group_or_tuple_expr(&mut self) -> Node<Expr> {
        let pos = self.current.pos.clone();
        self.step();
        match self.current.token_type {
            TokenType::RParen => {
                self.step();
                Node::new(Expr::Unit, pos)
            }
            _ => {
                let first_expr = self.parse_expr();
                match &self.current.token_type {
                    TokenType::RParen => {
                        self.step();
                        first_expr
                    }
                    TokenType::Comma => {
                        let mut exprs = vec![first_expr];
                        while self.current.token_type == TokenType::Comma {
                            self.step();
                            if self.current.token_type == TokenType::RParen {
                                break;
                            }
                            exprs.push(self.parse_expr());
                        }
                        if self.current.token_type != TokenType::RParen {
                            self.add_error(
                                self.pos(),
                                format!("expected ')', got {:?}", self.current.token_type),
                            );
                        } else {
                            self.step();
                        }
                        Node::new(Expr::Tuple(exprs), pos)
                    }
                    token_type => {
                        self.add_error(
                            self.pos(),
                            format!("expected ',' or ')', got {token_type:?}"),
                        );
                        Node::new(Expr::Error, pos)
                    }
                }
            }
        }
    }

    fn parse_block_expr(&mut self) -> Node<Expr> {
        let pos = self.current.pos.clone();
        self.step();
        let mut statements = Vec::<Node<Expr>>::new();
        match self.current.token_type {
            TokenType::RBrace => {
                self.step();
                Node::new(
                    Expr::Block {
                        statements,
                        expr: None,
                    },
                    pos,
                )
            }
            _ => loop {
                let _requires_semicolon = requires_semicolon(&self.current.token_type);
                statements.push(self.parse_statement());
                match &self.current.token_type {
                    TokenType::RBrace => {
                        self.step();
                        let expr = statements.pop().unwrap();
                        break Node::new(
                            Expr::Block {
                                statements,
                                expr: Some(Box::new(expr)),
                            },
                            pos,
                        );
                    }
                    TokenType::Semicolon => {
                        while self.current.token_type == TokenType::Semicolon {
                            self.step();
                        }
                        match &self.current.token_type {
                            TokenType::RBrace => {
                                self.step();
                                break Node::new(
                                    Expr::Block {
                                        statements,
                                        expr: None,
                                    },
                                    pos,
                                );
                            }
                            token_type @ TokenType::Eof => {
                                self.add_error(
                                    self.current.pos.clone(),
                                    format!("expected '}}', got {token_type:?}"),
                                );
                                break Node::new(Expr::Error, pos);
                            }
                            _ => {}
                        }
                    }
                    token_type => {
                        self.add_error(
                            self.current.pos.clone(),
                            format!("expected '}}' or ';', got {token_type:?}"),
                        );
                        break Node::new(Expr::Error, pos);
                    }
                }
            },
        }
    }

    fn parse_array_expr(&mut self) -> Node<Expr> {
        let pos = self.current.pos.clone();
        self.step();
        let mut exprs = Vec::<Node<Expr>>::new();
        match self.current.token_type {
            TokenType::RBracket => {
                self.step();
                Node::new(Expr::Array(exprs), pos)
            }
            _ => {
                exprs.push(self.parse_expr());
                loop {
                    match &self.current.token_type {
                        TokenType::RBracket => {
                            self.step();
                            break Node::new(Expr::Array(exprs), pos);
                        }
                        TokenType::Comma => {
                            self.step();
                            match self.current.token_type {
                                TokenType::RBracket => {
                                    self.step();
                                    break Node::new(Expr::Array(exprs), pos);
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            exprs.push(self.parse_expr());
                        }
                    }
                }
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

fn requires_semicolon(token_type: &TokenType) -> bool {
    match token_type {
        _ => true,
    }
}
