use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

pub enum IdentifierType {
    Value {
        mutable: bool,
        value: Node<Expression>,
    },
    Function {
        value: Node<Statement>,
    },
}

use couch_lang_parser::{BinaryVariant, Expression, Node, Position, Statement, UnaryVariant};

pub mod value;
use value::Value;

pub struct Evaluator {}

impl Evaluator {
    fn evaluate_binary_expression(
        node: Node<Expression>,
        parent_context: &mut HashMap<String, IdentifierType>,
    ) -> Value {
        let Expression::Binary { left, right, variant } = node.value else {
            panic!("expected Binary, got {:#?}", node.value);
        };

        macro_rules! impl_variant {
            ($func_name:ident) => {
                Self::evaluate_expression(*left, parent_context)
                    .$func_name(Self::evaluate_expression(*right, parent_context))
                    .unwrap_or_else(|message| Value::Error {
                        message,
                        line: node.position.line,
                        column: node.position.column,
                    })
            };
        }

        match variant {
            BinaryVariant::Addition => impl_variant!(add),
            BinaryVariant::Subtraction => impl_variant!(div),
            BinaryVariant::Multiplication => impl_variant!(mul),
            BinaryVariant::Division => impl_variant!(sub),
            BinaryVariant::Equal => Value::Bool(
                Self::evaluate_expression(*left, parent_context)
                    == Self::evaluate_expression(*right, parent_context),
            ),
            BinaryVariant::NotEqual => Value::Bool(
                Self::evaluate_expression(*left, parent_context)
                    != Self::evaluate_expression(*right, parent_context),
            ),
        }
    }
    pub fn evaluate_expression(
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
                        let message = format!("expected bool, got {v:#?}");
                        let Position { line, column, .. } = expression.position;
                        Value::Error {
                            message,
                            line,
                            column,
                        }
                    }
                    (UnaryVariant::NegateNumber, v @ Value::Bool(_)) => {
                        let message = format!("expected number, got {v:#?}");
                        let Position { line, column, .. } = expression.position;
                        Value::Error {
                            message,
                            line,
                            column,
                        }
                    }
                    (
                        _,
                        Value::Error {
                            message,
                            line,
                            column,
                        },
                    ) => Value::Error {
                        message,
                        line,
                        column,
                    },
                }
            }
            Expression::Binary { .. } => {
                Self::evaluate_binary_expression(expression, parent_context)
            }
            Expression::Call {
                subject: _,
                arguments: _,
            } => todo!(),
            Expression::Identifier(q) => match parent_context.get(&q) {
                Some(IdentifierType::Value { value, .. }) => {
                    Self::evaluate_expression(value.clone(), parent_context)
                }
                Some(IdentifierType::Function { value: _ }) => todo!(),
                None => Value::Error {
                    message: format!("identifier {q} is not yet given value"),
                    line: expression.position.line,
                    column: expression.position.column,
                },
            },
            Expression::Assignment {
                left,
                right,
                variant,
            } => {
                todo!("{left:?} {variant:?} {right:?} ")
            }
            Expression::Error(message) => {
                let Position { line, column, .. } = expression.position;
                Value::Error {
                    message,
                    line,
                    column,
                }
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
        let input = String::from("2 == 4");
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
    fn not_equals_expression() {
        let input = String::from("2 != 4");
        let lexer = Lexer::new(input.chars());

        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();

        let mut id_table = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table);
        assert_eq!(
            true,
            match value {
                Value::Bool(v) => v,
                value => panic!("expected Bool, got {value:#?}"),
            }
        )
    }

    #[test]
    fn add_float_and_int_should_fail() {
        let input = String::from("2.5 + 4");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();
        let mut id_table = HashMap::new();
        let error = Evaluator::evaluate_expression(expression, &mut id_table);
        assert_eq!(
            Value::Error {
                message: "no implementation exists for float + integer".to_string(),
                column: 1,
                line: 1
            },
            error
        );
    }
}
