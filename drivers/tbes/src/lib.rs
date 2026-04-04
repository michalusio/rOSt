#![no_std] // no standard library
#![no_main]
#![allow(incomplete_features)]
#![feature(
    generic_const_exprs,
    adt_const_params,
    option_into_flat_iter,
    iter_next_chunk,
    used_with_arg
)]

extern crate alloc;

use internal_utils::tag_store::Identity;

mod multi_value_index;
mod query;
mod tags;
pub use tags::init_tag_store;
