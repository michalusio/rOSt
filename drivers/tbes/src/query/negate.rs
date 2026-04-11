use internal_utils::tag_store::{
    BoolQueryExpression, BoolQueryExpressionType, IdentityQueryExpression, QueryExpression,
    U64QueryExpression, U64QueryExpressionType,
};

pub trait Negatable {
    fn negate(self) -> Self;
}

impl Negatable for BoolQueryExpression {
    fn negate(self) -> Self {
        Self {
            second: !self.second,
            ..self
        }
    }
}

impl Negatable for U64QueryExpressionType {
    fn negate(self) -> Self {
        match self {
            U64QueryExpressionType::EqualTo => U64QueryExpressionType::NotEqualTo,
            U64QueryExpressionType::NotEqualTo => U64QueryExpressionType::EqualTo,
            U64QueryExpressionType::LessThan => U64QueryExpressionType::GreaterThanOrEqualTo,
            U64QueryExpressionType::LessThanOrEqualTo => U64QueryExpressionType::GreaterThan,
            U64QueryExpressionType::GreaterThan => U64QueryExpressionType::LessThanOrEqualTo,
            U64QueryExpressionType::GreaterThanOrEqualTo => U64QueryExpressionType::LessThan,
        }
    }
}

impl Negatable for U64QueryExpression {
    fn negate(self) -> Self {
        Self {
            operation: self.operation.negate(),
            ..self
        }
    }
}

impl Negatable for IdentityQueryExpression {
    fn negate(self) -> Self {
        Self {
            operation: match self.operation {
                BoolQueryExpressionType::EqualTo => BoolQueryExpressionType::NotEqualTo,
                BoolQueryExpressionType::NotEqualTo => BoolQueryExpressionType::EqualTo,
            },
            ..self
        }
    }
}

impl Negatable for QueryExpression {
    fn negate(self) -> Self {
        match self {
            QueryExpression::Bool(bool_query_expression) => {
                QueryExpression::Bool(bool_query_expression.negate())
            }
            QueryExpression::U64(u64_query_expression) => {
                QueryExpression::U64(u64_query_expression.negate())
            }
            QueryExpression::Identity(identity_query_expression) => {
                QueryExpression::Identity(identity_query_expression.negate())
            }
        }
    }
}
