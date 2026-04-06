use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::tag_store::{BooleanTag, Identity, IntegerTag, RefTag};

#[derive(Clone)]
pub enum Query {
    And(Vec<Query>),
    Or(Vec<Query>),
    Not(Box<Query>),
    Binary(QueryExpression),
}

#[derive(Clone)]
pub enum QueryExpression {
    Bool(BoolQueryExpression),
    U64(U64QueryExpression),
    Identity(IdentityQueryExpression),
}

impl From<QueryExpression> for Query {
    fn from(value: QueryExpression) -> Self {
        Query::Binary(value)
    }
}

#[derive(Clone)]
pub struct BoolQueryExpression {
    pub first: Arc<dyn BooleanTag>,
    pub second: bool,
    pub operation: BoolQueryExpressionType,
}

impl From<BoolQueryExpression> for QueryExpression {
    fn from(value: BoolQueryExpression) -> Self {
        QueryExpression::Bool(value)
    }
}

#[derive(Clone)]
pub struct U64QueryExpression {
    pub first: Arc<dyn IntegerTag>,
    pub second: u64,
    pub operation: U64QueryExpressionType,
}

impl From<U64QueryExpression> for QueryExpression {
    fn from(value: U64QueryExpression) -> Self {
        QueryExpression::U64(value)
    }
}

#[derive(Clone)]
pub struct IdentityQueryExpression {
    pub first: Arc<dyn RefTag>,
    pub second: Identity,
    pub operation: BoolQueryExpressionType,
}

impl From<IdentityQueryExpression> for QueryExpression {
    fn from(value: IdentityQueryExpression) -> Self {
        QueryExpression::Identity(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoolQueryExpressionType {
    EqualTo,
    NotEqualTo,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum U64QueryExpressionType {
    EqualTo,
    NotEqualTo,
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
}
