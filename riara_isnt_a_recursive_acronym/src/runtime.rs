use create::pos::{Pos, ErrorType, Error, Node};
use create::parsed::{UnaryType, BinaryType, Expr};

#[derive(Debug)]
enum Value {
    Int(i64),
    String(String),
}

struct Evaluator {}

impl Evalutor {
    pub fn eval_expr(expr: Node<Expr>) -> Result<Value, Error> {
        let pos = expr.pos.clone();
        match expr.value {
            Expr::Error => panic!("illegal situation, evaluated ast must not contain errors"),
            Expr::Id(_) => todo!(),
            Expr::Int(v) => Ok(Value::Int(v)),
            Expr::String(v) => Ok(Value::String(v)),
            Expr::Unary {
                unary_type,
                subject,
            } => {
                let value = eval_expr(*subject)?;
                match (&unary_type, value) {
                    (UnaryType::Plus, v @ Value::Int(_)) => Ok(v),
                    (UnaryType::Negate, Value::Int(v)) => Ok(Value::Int(-v)),
                    (unary_type, _) => Err(self.error(pos, format!("invalid {unary_type} operation"))),
                }
            }
            Expr::Binary {
                binary_type,
                left,
                right,
            } => {
                let left = eval_expr(*left)?;
                let right = eval_expr(*right)?;
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
                    (binary_type, _, _) => Err(self.error(pos, format!("invalid {binary_type} operation"))),
                }
            }
        }
    }

    fn error(&self, pos: Pos, message: String) -> Error {
        Error { error_type: ErrorType::Runtime, pos, message }
    }
}
