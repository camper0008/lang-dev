#[macro_export]
macro_rules! try_peek_or_error {
    (parser: $self:ident, expect: $expected:ident, error: $error_type:ident::Error) => {{
        let Some(next) = $self.iter.peek() else {
            let message = format!("expected '{:?}', got end of file", TokenVariant::$expected);
            let position = Position {
                index: $self.text.len(),
                line: $self.text.split('\n').count(),
                column: $self.text.split('\n').last().unwrap_or("").len(),
            };
            return Self::node($error_type::Error(message), position);
        };
        next
    }};
    (parser: $self:ident, error: $error_type:ident::Error) => {{
        let Some(next) = $self.iter.peek() else {
                                let message = format!("expected token, got end of file");
                                let position = Position {
                                        index: $self.text.len(),
                                        line: $self.text.split('\n').count(),
                                        column: $self.text.split('\n').last().unwrap_or("").len(),
                                    };
                                    return Self::node($error_type::Error(message), position);
                                };
        next
    }};
}

#[macro_export]
macro_rules! assert_equal_variant {
    ($token:ident == $should_be_variant:ident, error: $error_type:ident::Error) => {
        if !matches!(&$token.variant, TokenVariant::$should_be_variant) {
            let message = format!(
                "expected '{}', got '{:?}'",
                stringify!($should_be_variant),
                $token.variant,
            );
            return Self::node($error_type::Error(message), $token.into());
        }
    };
}
