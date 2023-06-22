use crate::pos::Node;

#[derive(Debug)]
pub enum UnaryType {
    Plus,
    Negate,
}

#[derive(Debug)]
pub enum BinaryType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Expr {
    Error,
    Id(String),
    Int(i64),
    String(String),
    Unary {
        unary_type: UnaryType,
        subject: Box<Node<Expr>>,
    },
    Binary {
        binary_type: BinaryType,
        left: Box<Node<Expr>>,
        right: Box<Node<Expr>>,
    },
}
