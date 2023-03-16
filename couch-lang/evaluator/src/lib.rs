use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

enum IdentifierType {
    Value {
        mutable: bool,
        value: Node<Expression>,
    },
    Function {
        value: Node<Statement>,
    },
}

use couch_lang_parser::{BinaryVariant, Expression, Node, Statement, UnaryVariant};

pub mod value;
use value::Value;

pub struct Evaluator {}

impl Evaluator {
    fn evaluate_expression(
        expression: Node<Expression>,
        parent_context: &mut HashMap<String, IdentifierType>,
    ) -> Value {
        match expression.value {
            Expression::Integer(v) => Value::Integer(v),
            Expression::Float(v) => Value::Float(v),
            Expression::Unary { subject, variant } => {
                let evaluated_subject = Self::evaluate_expression(*subject, parent_context);
                match (variant, evaluated_subject) {
                    (UnaryVariant::NegateNumber, Value::Integer(v)) => Value::Integer(-v),
                    (UnaryVariant::NegateNumber, Value::Float(v)) => Value::Float(-v),
                    (UnaryVariant::NegateBool, Value::Bool(v)) => Value::Bool(!v),
                    (UnaryVariant::NegateBool, v @ (Value::Integer(_) | Value::Float(_))) => {
                        panic!("expected bool, got {v:#?}")
                    }
                    (UnaryVariant::NegateNumber, v @ Value::Bool(_)) => {
                        panic!("expected number, got {v:#?}")
                    }
                }
            }
            Expression::Binary {
                left,
                right,
                variant,
            } => match variant {
                BinaryVariant::Addition => Self::evaluate_expression(*left, parent_context)
                    .add(Self::evaluate_expression(*right, parent_context))
                    .unwrap(),
                BinaryVariant::Subtraction => Self::evaluate_expression(*left, parent_context)
                    .sub(Self::evaluate_expression(*right, parent_context))
                    .unwrap(),
                BinaryVariant::Multiplication => Self::evaluate_expression(*left, parent_context)
                    .mul(Self::evaluate_expression(*right, parent_context))
                    .unwrap(),
                BinaryVariant::Division => Self::evaluate_expression(*left, parent_context)
                    .div(Self::evaluate_expression(*right, parent_context))
                    .unwrap(),
                BinaryVariant::Equal => Value::Bool(
                    Self::evaluate_expression(*left, parent_context)
                        == Self::evaluate_expression(*right, parent_context),
                ),
                BinaryVariant::NotEqual => Value::Bool(
                    Self::evaluate_expression(*left, parent_context)
                        == Self::evaluate_expression(*right, parent_context),
                ),
            },
            Expression::Call { subject, arguments } => todo!(),
            Expression::Identifier(q) => match parent_context
                .get(&q)
                .expect("identifier {q} is not initialized")
            {
                IdentifierType::Value { value, .. } => {
                    Self::evaluate_expression(value.clone(), parent_context)
                }
                _ => todo!(),
            },
            Expression::Assignment {
                left,
                right,
                variant,
            } => {
                todo!("{left:?} {variant:?} {right:?} ")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use couch_lang_lexer::Lexer;
    use couch_lang_parser::Parser;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn add_expression() {
        let input = String::from("2 + 4");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();
        let mut id_table = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table);
        assert_eq!(
            6,
            match value {
                Value::Integer(v) => v,
                value => panic!("expected Integer, got {value:#?}"),
            }
        )
    }

    #[test]
    fn identifier() {
        let input = String::from("a 5");
        let lexer = Lexer::new(input.chars());

        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();
        let value = parser.parse_expression();

        let mut id_table = HashMap::from([(
            "a".to_string(),
            IdentifierType::Value {
                value,
                mutable: false,
            },
        )]);
        let value = Evaluator::evaluate_expression(expression, &mut id_table);
        assert_eq!(
            5,
            match value {
                Value::Integer(v) => v,
                value => panic!("expected Bool, got {value:#?}"),
            }
        )
    }

    #[test]
    fn equals_expression() {
        let input = String::from("2 != 4");
        let lexer = Lexer::new(input.chars());

        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();

        let mut id_table = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table);
        assert_eq!(
            false,
            match value {
                Value::Bool(v) => v,
                value => panic!("expected Bool, got {value:#?}"),
            }
        )
    }

    #[test]
    #[should_panic(expected = "no implementation exists for float + integer")]
    fn add_float_and_int_should_fail() {
        let input = String::from("2.5 + 4");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();
        let mut id_table = HashMap::new();
        Evaluator::evaluate_expression(expression, &mut id_table);
        unreachable!("should fail");
    }
}
