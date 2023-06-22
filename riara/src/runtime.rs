use crate::parsed::{BinaryType, Expr, UnaryType};
use crate::pos::{Error, ErrorType, Node, Pos};

#[derive(Debug)]
pub enum Value {
    Unit,
    Int(i64),
    String(String),
    Bool(bool),
    Tuple(Vec<Value>),
    Array(Vec<Value>),
}

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval_expr(&mut self, expr: &Node<Expr>) -> Result<Value, Error> {
        let pos = expr.pos.clone();
        match &expr.value {
            Expr::Error => panic!("illegal situation, evaluated ast must not contain errors"),
            Expr::Unit => Ok(Value::Unit),
            Expr::Id(_) => todo!(),
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::String(v) => Ok(Value::String(v.clone())),
            Expr::Bool(v) => Ok(Value::Bool(*v)),
            Expr::Tuple(exprs) => Ok(Value::Tuple(
                exprs
                    .iter()
                    .map(|expr| self.eval_expr(expr))
                    .collect::<Result<_, _>>()?,
            )),
            Expr::Array(exprs) => Ok(Value::Array(
                exprs
                    .iter()
                    .map(|expr| self.eval_expr(expr))
                    .collect::<Result<_, _>>()?,
            )),
            Expr::Block { statements, expr } => {
                for statement in statements {
                    let _ = self.eval_expr(&statement)?;
                }
                match expr {
                    Some(expr) => self.eval_expr(&expr),
                    None => Ok(Value::Unit),
                }
            }
            Expr::Unary {
                unary_type,
                subject,
            } => {
                let value = self.eval_expr(&subject)?;
                match (&unary_type, value) {
                    (UnaryType::Plus, v @ Value::Int(_)) => Ok(v),
                    (UnaryType::Negate, Value::Int(v)) => Ok(Value::Int(-v)),
                    (unary_type, _) => {
                        Err(self.error(pos, format!("invalid {unary_type:?} operation")))
                    }
                }
            }
            Expr::Binary {
                binary_type,
                left,
                right,
            } => {
                let left = self.eval_expr(&left)?;
                let right = self.eval_expr(&right)?;
                match (&binary_type, left, right) {
                    (BinaryType::Add, Value::Int(left), Value::Int(right)) => {
                        Ok(Value::Int(left + right))
                    }
                    (BinaryType::Subtract, Value::Int(left), Value::Int(right)) => {
                        Ok(Value::Int(left - right))
                    }
                    (BinaryType::Multiply, Value::Int(left), Value::Int(right)) => {
                        Ok(Value::Int(left * right))
                    }
                    (BinaryType::Divide, Value::Int(left), Value::Int(right)) => {
                        Ok(Value::Int(left / right))
                    }
                    (BinaryType::Add, Value::String(left), Value::String(right)) => {
                        Ok(Value::String(left + &right))
                    }
                    (binary_type, _, _) => {
                        Err(self.error(pos, format!("invalid {binary_type:?} operation")))
                    }
                }
            }
        }
    }

    fn error(&self, pos: Pos, message: String) -> Error {
        Error {
            error_type: ErrorType::Runtime,
            pos,
            message,
        }
    }
}
