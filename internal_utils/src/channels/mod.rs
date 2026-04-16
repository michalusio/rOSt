mod cache_padded;

pub use cache_padded::CachePadded;
pub mod spmc;
pub mod spsc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChannelSizeHint {
    /// A small channel, usually for passing integers or references
    Small,
    /// A large channel, used for passing bigger data objects
    Large,
}
