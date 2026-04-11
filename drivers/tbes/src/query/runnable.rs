use core::iter;

use alloc::format;
use alloc::vec;
use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};
use internal_utils::tag_store::IdentityQueryExpression;
use internal_utils::tag_store::{
    BoolQueryExpression, BoolQueryExpressionType, Query, QueryExpression, U64QueryExpression,
    U64QueryExpressionType,
};

use crate::query::negate::Negatable;
use crate::{Identity, query::query_context::QueryContext};

pub trait Runnable {
    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity>;
    /// Rewrites self into the Conjunctive normal form query
    fn normalize(self) -> Query;
}

impl Runnable for Query {
    fn normalize(self) -> Query {
        match self {
            Query::Or(conditions) => {
                let normalized_items: Vec<Query> =
                    conditions.into_iter().map(Runnable::normalize).collect();
                let (ors, mut rest): (Vec<_>, Vec<_>) = normalized_items
                    .into_iter()
                    .partition(|i| matches!(i, Query::Or(_)));
                rest.extend(ors.into_iter().flat_map(|a| match a {
                    Query::Or(c) => c,
                    _ => unreachable!(),
                }));

                let exploded = rest
                    .into_iter()
                    .map(|a| match a {
                        Query::And(c) => c,
                        _ => {
                            vec![a]
                        }
                    })
                    .fold(vec![], |curr: Vec<Vec<Query>>, next| {
                        let mut result: Vec<Vec<Query>> = vec![];
                        for n in next {
                            if curr.is_empty() {
                                result.push(iter::once(n).collect());
                            } else {
                                for c in &curr {
                                    result.push(c.iter().chain(iter::once(&n)).cloned().collect());
                                }
                            }
                        }
                        result
                    })
                    .into_iter()
                    .map(Query::Or)
                    .collect();

                Query::And(exploded)
            }
            Query::And(conditions) => {
                let (ands, mut rest): (Vec<_>, Vec<_>) = conditions
                    .into_iter()
                    .map(Runnable::normalize)
                    .partition(|i| matches!(*i, Query::And(_)));
                rest.extend(ands.into_iter().flat_map(|a| match a {
                    Query::And(c) => c,
                    _ => unreachable!(),
                }));
                Query::And(rest)
            }
            Query::Not(term) => match *term {
                Query::Not(t) => t.normalize(),

                Query::Or(conditions) => Query::And(
                    conditions
                        .into_iter()
                        .map(|c| Query::Not(Box::new(c)).normalize())
                        .collect(),
                ),

                Query::And(conditions) => Query::Or(
                    conditions
                        .into_iter()
                        .map(|c| Query::Not(Box::new(c)).normalize())
                        .collect(),
                ),

                Query::Binary(expression) => Query::Binary(expression.negate()),
            },
            Query::Binary(expression) => expression.normalize(),
        }
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        match self {
            Query::And(items) => reduce_down(items, query_writer, "NestedLoop", &|a, b| {
                BTreeSet::from_iter(a.intersection(b).copied())
            }),
            Query::Or(items) => reduce_down(items, query_writer, "Concatenate", &|a, b| {
                BTreeSet::from_iter(a.union(b).copied())
            }),
            Query::Binary(expression) => expression.run(query_writer),
            Query::Not(_) => {
                panic!("Query negations are not runnable - normalize the query first!")
            }
        }
    }
}

fn reduce_down(
    children: &[Query],
    query_context: &mut QueryContext,
    name: &'static str,
    reducer: &impl Fn(&BTreeSet<Identity>, &BTreeSet<Identity>) -> BTreeSet<Identity>,
) -> BTreeSet<Identity> {
    match children.len() {
        0 => BTreeSet::new(),
        1 => children[0].run(query_context),
        _ => {
            query_context.open_section(name);
            let (left, right) = children.split_at(children.len() / 2);
            let set = reducer(
                &reduce_down(left, query_context, name, reducer),
                &reduce_down(right, query_context, name, reducer),
            );
            query_context.close_section();
            set
        }
    }
}

impl Runnable for QueryExpression {
    fn normalize(self) -> Query {
        Query::Binary(self)
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        match self {
            QueryExpression::Bool(bool_query) => bool_query.run(query_writer),
            QueryExpression::U64(u64_query) => u64_query.run(query_writer),
            QueryExpression::Identity(identity_query) => identity_query.run(query_writer),
        }
    }
}

impl Runnable for BoolQueryExpression {
    fn normalize(self) -> Query {
        Query::Binary(QueryExpression::Bool(self))
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        let value = (self.operation == BoolQueryExpressionType::EqualTo) ^ !self.second;
        query_writer.item_vec([self.first.name(), "=", if value { "true" } else { "false" }]);
        self.first.get_identities(value)
    }
}

impl Runnable for U64QueryExpression {
    fn normalize(self) -> Query {
        Query::Binary(QueryExpression::U64(self))
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        query_writer.item_vec([
            self.first.name(),
            match self.operation {
                U64QueryExpressionType::EqualTo => "=",
                U64QueryExpressionType::NotEqualTo => "!=",
                U64QueryExpressionType::LessThan => "<",
                U64QueryExpressionType::LessThanOrEqualTo => "<=",
                U64QueryExpressionType::GreaterThan => ">",
                U64QueryExpressionType::GreaterThanOrEqualTo => ">=",
            },
            &format!("{}", self.second),
        ]);
        self.first.get_identities(self.second, self.operation)
    }
}

impl Runnable for IdentityQueryExpression {
    fn normalize(self) -> Query {
        Query::Binary(QueryExpression::Identity(self))
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        query_writer.item_vec([
            self.first.name(),
            match self.operation {
                BoolQueryExpressionType::EqualTo => "=",
                BoolQueryExpressionType::NotEqualTo => "!=",
            },
            &format!("{}", self.second),
        ]);
        self.first.get_identities(
            self.second,
            self.operation == BoolQueryExpressionType::NotEqualTo,
        )
    }
}
