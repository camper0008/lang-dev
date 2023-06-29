use crate::parsed::{BinaryType, Expr, UnaryType};
use crate::pos::{Error, ErrorType, Node, Pos};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Char(char),
    String(String),
    Bool(bool),
    Tuple(Vec<Value>),
    Array(Vec<Value>),
}

impl Value {
    pub fn type_string(&self) -> String {
        match self {
            Value::Unit => "()",
            Value::Int(_) => "int",
            Value::Char(_) => "char",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Tuple(_) => "tuple",
            Value::Array(_) => "array",
        }
        .to_string()
    }
}

struct Symbol {
    stack_depth: usize,
    id: String,
    value: Value,
}

pub struct Evaluator {
    stack_depth: usize,
    symbols: Vec<Symbol>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            stack_depth: 0,
            symbols: Vec::new(),
        }
    }

    pub fn eval_expr(&mut self, expr: &Node<Expr>) -> Result<Value, Error> {
        let pos = expr.pos.clone();
        match &expr.value {
            Expr::Error => unreachable!("illegal situation, evaluated ast must not contain errors"),
            Expr::Unit => Ok(Value::Unit),
            Expr::Id(id) => self
                .find_symbol(id.clone())
                .map(|s| s.value.clone())
                .ok_or_else(|| self.error(pos, format!("undefined symbol '{id}'"))),
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
                self.enter_scope();
                for statement in statements {
                    let _ = self.eval_expr(&statement)?;
                }
                let result = match expr {
                    Some(expr) => self.eval_expr(&expr),
                    None => Ok(Value::Unit),
                };
                self.leave_scope();
                result
            }
            Expr::If {
                condition,
                truthy,
                falsy,
            } => {
                let condition = self.eval_expr(condition)?;
                match (&condition, falsy) {
                    (Value::Bool(true), _) => self.eval_expr(truthy),
                    (Value::Bool(false), Some(falsy)) => self.eval_expr(falsy),
                    (Value::Bool(false), None) => Ok(Value::Unit),
                    _ => Err(self.error(
                        pos,
                        format!("expected bool, got {}", condition.type_string()),
                    )),
                }
            }
            Expr::Index { subject, value } => {
                let value = self.eval_expr(value)?;
                let subject = self.eval_expr(subject)?;
                match (subject, value) {
                    (Value::String(v), Value::Int(i)) => {
                        if let Some(c) = v.chars().nth(i as usize) {
                            Ok(Value::Char(c))
                        } else {
                            Err(self.error(pos, format!("index overflow")))
                        }
                    }
                    (Value::Tuple(vs) | Value::Array(vs), Value::Int(i)) => {
                        if let Some(v) = vs.get(i as usize) {
                            Ok(v.clone())
                        } else {
                            Err(self.error(pos, format!("index overflow")))
                        }
                    }
                    (subject, value) => Err(self.error(
                        pos,
                        format!(
                            "cannot index into {} with {}",
                            subject.type_string(),
                            value.type_string()
                        ),
                    )),
                }
            }
            Expr::Call {
                subject: _,
                arguments: _,
            } => todo!(),
            Expr::Unary {
                unary_type,
                subject,
            } => {
                let value = self.eval_expr(&subject)?;
                match (&unary_type, value) {
                    (UnaryType::Plus, v @ Value::Int(_)) => Ok(v),
                    (UnaryType::Negate, Value::Int(v)) => Ok(Value::Int(-v)),
                    (UnaryType::Not, Value::Bool(v)) => Ok(Value::Bool(!v)),
                    (unary_type, _) => {
                        Err(self.error(pos, format!("invalid unary {unary_type:?} operation")))
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
                    (BinaryType::Or, Value::Bool(left), Value::Bool(right)) => {
                        Ok(Value::Bool(left || right))
                    }
                    (BinaryType::And, Value::Bool(left), Value::Bool(right)) => {
                        Ok(Value::Bool(left && right))
                    }
                    (BinaryType::Includes, left, Value::Array(right)) => {
                        Ok(Value::Bool(match right.iter().find(|v| **v == left) {
                            Some(_) => true,
                            None => false,
                        }))
                    }
                    (BinaryType::Excludes, left, Value::Array(right)) => {
                        Ok(Value::Bool(match right.iter().find(|v| **v == left) {
                            Some(_) => false,
                            None => true,
                        }))
                    }
                    (binary_type, _, _) => {
                        Err(self.error(pos, format!("invalid binary {binary_type:?} operation")))
                    }
                }
            }
            Expr::Let { id, value } => {
                let value = self.eval_expr(value)?;
                self.define_symbol(id.clone(), value);
                Ok(Value::Unit)
            }
        }
    }

    fn enter_scope(&mut self) {
        self.stack_depth += 1;
    }

    fn leave_scope(&mut self) {
        self.stack_depth -= 1;
        self.symbols.retain(|s| s.stack_depth <= self.stack_depth);
    }

    fn define_symbol(&mut self, id: String, value: Value) {
        if let Some(mut s) = self.symbols.iter_mut().rfind(|s| s.id == id) {
            if s.stack_depth == self.stack_depth {
                s.value = value;
            } else {
                self.symbols.push(Symbol {
                    id,
                    value,
                    stack_depth: self.stack_depth,
                })
            }
        } else {
            self.symbols.push(Symbol {
                id,
                value,
                stack_depth: self.stack_depth,
            })
        }
    }

    fn find_symbol(&self, id: String) -> Option<&Symbol> {
        self.symbols.iter().rfind(|s| s.id == id)
    }

    fn error(&self, pos: Pos, message: String) -> Error {
        Error {
            error_type: ErrorType::Runtime,
            pos,
            message,
        }
    }
}
