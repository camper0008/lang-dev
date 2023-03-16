use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(_) => f.write_str("integer"),
            Value::Float(_) => f.write_str("float"),
            Value::Bool(_) => f.write_str("bool"),
        }
    }
}

macro_rules! implement_operator {
    // The `ident` designator is used for variable/function names.
    // The `tt` (token tree) designator is used for
    // operators and tokens.
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

implement_operator!(Add, add, +);
implement_operator!(Sub, sub, -);
implement_operator!(Mul, mul, *);
implement_operator!(Div, div, /);
