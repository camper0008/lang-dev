use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Error {
        message: String,
        line: usize,
        column: usize,
    },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(_) => f.write_str("integer"),
            Value::Float(_) => f.write_str("float"),
            Value::Bool(_) => f.write_str("bool"),
            Value::Error { .. } => f.write_str("error"),
        }
    }
}

macro_rules! implement_operator {
    ($func_trait:ident, $func_name:ident, $op:tt) => {
        impl $func_trait for Value {
            type Output = Result<Value, String>;
            fn $func_name(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a $op b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a $op b)),
                    (a, b) => Err(format!("no implementation exists for {a} {} {b}", stringify!($op))),
                }
            }
        }
    };
}

macro_rules! implement_operator_assign {
    ($func_trait:ident, $func_name:ident, $op:tt) => {
        impl $func_trait for Value {
            fn $func_name(&mut self, rhs: Self) {
                match (self, rhs) {
                    (Value::Integer(a), Value::Integer(b)) => {*a $op b},
                    (Value::Float(a), Value::Float(b)) => {*a $op b},
                    (a, b) => panic!("no implementation exists for {a} {}= {b}", stringify!($op)),
                }
            }
        }
    };
}

implement_operator!(Add, add, +);
implement_operator!(Sub, sub, -);
implement_operator!(Mul, mul, *);
implement_operator!(Div, div, /);

implement_operator_assign!(AddAssign, add_assign, +=);
implement_operator_assign!(SubAssign, sub_assign, -=);
implement_operator_assign!(MulAssign, mul_assign, *=);
implement_operator_assign!(DivAssign, div_assign, /=);
