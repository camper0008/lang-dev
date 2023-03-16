use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

pub enum IdentifierType {
    Value { mutable: bool, value: Value },
    Function { value: Node<Statement> },
}

use couch_lang_parser::{
    AssignmentVariant, BinaryVariant, Expression, Node, Position, Statement, UnaryVariant,
};

pub mod value;
use value::Value;

pub struct Evaluator {}

impl Evaluator {
    fn evaluate_statements(statements: Vec<Node<Statement>>) -> Option<Value> {
        let mut outer_context = HashMap::new();
        let mut inner_context = HashMap::new();
        statements.into_iter().find_map(|statement| {
            Self::evaluate_statement(statement, &mut outer_context, &mut inner_context)
        })
    }
    fn evaluate_statement(
        node: Node<Statement>,
        outer_context: &mut HashMap<String, IdentifierType>,
        inner_context: &mut HashMap<String, IdentifierType>,
    ) -> Option<Value> {
        match node.value {
            Statement::Let {
                mutable,
                identifier,
                value,
            } => {
                let Expression::Identifier(identifier) = identifier.value else {
                    todo!("handle error");
                };
                let value = Self::evaluate_expression(*value, outer_context, inner_context);
                inner_context.insert(identifier, IdentifierType::Value { mutable, value });
                None
            }
            Statement::Return(_) => todo!(),
            Statement::Error(_) => todo!(),
            Statement::Assignment {
                left,
                right,
                variant,
            } => {
                let Expression::Identifier(identifier) = left.value else {
                    todo!("handle error");
                };
                let right = Evaluator::evaluate_expression(*right, outer_context, inner_context);
                let Some(identifier_ref) = inner_context
                    .get_mut(&identifier)
                    .or(outer_context.get_mut(&identifier)) else {
                        return Some(Value::Error { message: format!("identifier {identifier} not defined"), line: node.position.line, column: node.position.column });
                };
                match identifier_ref {
                    IdentifierType::Value { mutable, value } => {
                        if *mutable {
                            match variant {
                                AssignmentVariant::Base => *value = right,
                                AssignmentVariant::Addition => *value += right,
                                AssignmentVariant::Subtraction => *value -= right,
                                AssignmentVariant::Multiplication => *value *= right,
                                AssignmentVariant::Division => *value /= right,
                            };
                            None
                        } else {
                            Some(Value::Error {
                                message: format!("identifier {identifier} is not mutable"),
                                line: node.position.line,
                                column: node.position.column,
                            })
                        }
                    }
                    IdentifierType::Function { .. } => Some(Value::Error {
                        message: format!("function definitions are not mutable"),
                        line: node.position.line,
                        column: node.position.column,
                    }),
                }
            }
            Statement::Expression(expression) => Some(Self::evaluate_expression(
                expression,
                outer_context,
                inner_context,
            )),
        }
    }
    fn evaluate_binary_expression(
        node: Node<Expression>,
        outer_context: &mut HashMap<String, IdentifierType>,
        inner_context: &mut HashMap<String, IdentifierType>,
    ) -> Value {
        let Expression::Binary { left, right, variant } = node.value else {
            panic!("expected Binary, got {:#?}", node.value);
        };

        macro_rules! impl_variant {
            ($func_name:ident) => {
                Self::evaluate_expression(*left, outer_context, inner_context)
                    .$func_name(Self::evaluate_expression(
                        *right,
                        outer_context,
                        inner_context,
                    ))
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
                Self::evaluate_expression(*left, outer_context, inner_context)
                    == Self::evaluate_expression(*right, outer_context, inner_context),
            ),
            BinaryVariant::NotEqual => Value::Bool(
                Self::evaluate_expression(*left, outer_context, inner_context)
                    != Self::evaluate_expression(*right, outer_context, inner_context),
            ),
        }
    }
    pub fn evaluate_expression(
        expression: Node<Expression>,
        outer_context: &mut HashMap<String, IdentifierType>,
        inner_context: &mut HashMap<String, IdentifierType>,
    ) -> Value {
        match expression.value {
            Expression::Integer(v) => Value::Integer(v),
            Expression::Float(v) => Value::Float(v),
            Expression::Unary { subject, variant } => {
                let evaluated_subject =
                    Self::evaluate_expression(*subject, outer_context, inner_context);
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
                Self::evaluate_binary_expression(expression, outer_context, inner_context)
            }
            Expression::Call {
                subject: _,
                arguments: _,
            } => todo!(),
            Expression::Identifier(q) => match inner_context.get(&q).or(outer_context.get(&q)) {
                Some(IdentifierType::Value { value, .. }) => value.clone(),
                Some(IdentifierType::Function { value: _ }) => todo!(),
                None => Value::Error {
                    message: format!("identifier {q} is not yet given value"),
                    line: expression.position.line,
                    column: expression.position.column,
                },
            },
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
        let mut id_table_0 = HashMap::new();
        let mut id_table_1 = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table_0, &mut id_table_1);
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
        let input = String::from("a");
        let lexer = Lexer::new(input.chars());

        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();

        let mut id_table_0 = HashMap::from([(
            "a".to_string(),
            IdentifierType::Value {
                value: Value::Integer(5),
                mutable: false,
            },
        )]);
        let mut id_table_1 = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table_0, &mut id_table_1);
        assert_eq!(
            5,
            match value {
                Value::Integer(v) => v,
                value => panic!("expected Integer, got {value:#?}"),
            }
        )
    }

    #[test]
    fn equals_expression() {
        let input = String::from("2 == 4");
        let lexer = Lexer::new(input.chars());

        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let expression = parser.parse_expression();

        let mut id_table_0 = HashMap::new();
        let mut id_table_1 = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table_0, &mut id_table_1);
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

        let mut id_table_0 = HashMap::new();
        let mut id_table_1 = HashMap::new();
        let value = Evaluator::evaluate_expression(expression, &mut id_table_0, &mut id_table_1);
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
        let mut id_table_0 = HashMap::new();
        let mut id_table_1 = HashMap::new();
        let error = Evaluator::evaluate_expression(expression, &mut id_table_0, &mut id_table_1);
        assert_eq!(
            Value::Error {
                message: "no implementation exists for float + integer".to_string(),
                column: 1,
                line: 1
            },
            error
        );
    }

    #[test]
    fn evaluate_let_chain() {
        let input = String::from("let mut a = 5; a += 5; a;");
        let lexer = Lexer::new(input.chars());
        let mut parser = Parser::new(lexer.into_iter(), input.clone());
        let statements = parser.parse_statements();
        let error = Evaluator::evaluate_statements(statements);
        assert_eq!(Some(Value::Integer(10)), error);
    }
}
