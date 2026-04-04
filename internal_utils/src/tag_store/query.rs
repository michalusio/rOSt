use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::tag_store::{BooleanTag, Identity, IntegerTag, RefTag};

#[derive(Clone)]
pub enum Query {
    And(Vec<Query>),
    Or(Vec<Query>),
    Not(Box<Query>),
    Binary(BinaryQueryExpression),
}

#[derive(Clone)]
pub enum BinaryQueryExpression {
    Bool(BinaryBoolQueryExpression),
    U64(BinaryU64QueryExpression),
    Identity(BinaryIdentityQueryExpression),
}

impl From<BinaryQueryExpression> for Query {
    fn from(value: BinaryQueryExpression) -> Self {
        Query::Binary(value)
    }
}

#[derive(Clone)]
pub struct BinaryBoolQueryExpression {
    pub first: Arc<dyn BooleanTag>,
    pub second: bool,
    pub operation: BinaryBoolQueryExpressionType,
}

impl From<BinaryBoolQueryExpression> for BinaryQueryExpression {
    fn from(value: BinaryBoolQueryExpression) -> Self {
        BinaryQueryExpression::Bool(value)
    }
}

#[derive(Clone)]
pub struct BinaryU64QueryExpression {
    pub first: Arc<dyn IntegerTag>,
    pub second: u64,
    pub operation: BinaryU64QueryExpressionType,
}

impl From<BinaryU64QueryExpression> for BinaryQueryExpression {
    fn from(value: BinaryU64QueryExpression) -> Self {
        BinaryQueryExpression::U64(value)
    }
}

#[derive(Clone)]
pub struct BinaryIdentityQueryExpression {
    pub first: Arc<dyn RefTag>,
    pub second: Identity,
    pub operation: BinaryBoolQueryExpressionType,
}

impl From<BinaryIdentityQueryExpression> for BinaryQueryExpression {
    fn from(value: BinaryIdentityQueryExpression) -> Self {
        BinaryQueryExpression::Identity(value)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryBoolQueryExpressionType {
    EqualTo,
    NotEqualTo,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryU64QueryExpressionType {
    EqualTo,
    NotEqualTo,
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
}
