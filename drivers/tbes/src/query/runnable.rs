use core::iter;

use alloc::vec;
use alloc::{boxed::Box, collections::btree_set::BTreeSet, vec::Vec};
use internal_utils::tag_store::{
    BinaryBoolQueryExpression, BinaryBoolQueryExpressionType, BinaryQueryExpression, Query,
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
            Query::And(items) => {
                if items.len() == 1 {
                    items[0].run(query_writer)
                } else {
                    query_writer.open_section("SeriesNestedLoop");
                    let mut pairs = items.iter();
                    let mut processed = vec![];
                    'outer: loop {
                        match pairs.next_chunk::<2>() {
                            Ok([a, b]) => {
                                query_writer.open_section("NestedLoop");
                                processed.push(BTreeSet::from_iter(
                                    a.run(query_writer)
                                        .intersection(&b.run(query_writer))
                                        .copied(),
                                ));
                                query_writer.close_section();
                            }
                            Err(mut rest) => {
                                if let Some(rest) = rest.next() {
                                    processed.push(rest.run(query_writer));
                                }
                                break 'outer;
                            }
                        }
                    }
                    query_writer.close_section();
                    processed
                        .into_iter()
                        .reduce(|a, b| BTreeSet::from_iter(a.intersection(&b).copied()))
                        .unwrap_or_default()
                }
            }
            Query::Or(items) => {
                if items.len() == 1 {
                    items[0].run(query_writer)
                } else {
                    query_writer.open_section("SeriesConcatenate");
                    let mut pairs = items.iter();
                    let mut processed = vec![];
                    'outer: loop {
                        match pairs.next_chunk::<2>() {
                            Ok([a, b]) => {
                                query_writer.open_section("Concatenate");
                                processed.push(BTreeSet::from_iter(
                                    a.run(query_writer).union(&b.run(query_writer)).copied(),
                                ));
                                query_writer.close_section();
                            }
                            Err(mut rest) => {
                                if let Some(rest) = rest.next() {
                                    processed.push(rest.run(query_writer));
                                }
                                break 'outer;
                            }
                        }
                    }
                    query_writer.close_section();
                    processed
                        .into_iter()
                        .reduce(|a, b| BTreeSet::from_iter(a.union(&b).copied()))
                        .unwrap_or_default()
                }
            }
            Query::Binary(expression) => expression.run(query_writer),
            Query::Not(_) => {
                panic!("Query negations are not runnable - normalize the query first!")
            }
        }
    }
}

impl Runnable for BinaryQueryExpression {
    fn normalize(self) -> Query {
        Query::Binary(self)
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        match self {
            BinaryQueryExpression::Bool(bool_query) => bool_query.run(query_writer),
            BinaryQueryExpression::U64(u64_query) => todo!(),
            BinaryQueryExpression::Identity(identity_query) => todo!(),
        }
    }
}

impl Runnable for BinaryBoolQueryExpression {
    fn normalize(self) -> Query {
        Query::Binary(BinaryQueryExpression::Bool(self))
    }

    fn run(&self, query_writer: &mut QueryContext) -> BTreeSet<Identity> {
        let value = (self.operation == BinaryBoolQueryExpressionType::EqualTo) ^ !self.second;
        query_writer.item_vec([self.first.name(), "=", if value { "true" } else { "false" }]);
        self.first.get_identities(value)
    }
}
