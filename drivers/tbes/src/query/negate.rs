use internal_utils::tag_store::{BinaryBoolQueryExpression, BinaryQueryExpression};

pub trait Negatable {
    fn negate(self) -> Self;
}

impl Negatable for BinaryBoolQueryExpression {
    fn negate(self) -> Self {
        Self {
            second: !self.second,
            ..self
        }
    }
}

impl Negatable for BinaryQueryExpression {
    fn negate(self) -> Self {
        match self {
            BinaryQueryExpression::Bool(binary_bool_query_expression) => {
                BinaryQueryExpression::Bool(binary_bool_query_expression.negate())
            }
            BinaryQueryExpression::U64(binary_u64_query_expression) => {
                todo!()
            }
            BinaryQueryExpression::Identity(binary_identity_query_expression) => {
                todo!()
            }
        }
    }
}
