#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// Re-export `kona-derive` since its types are used in derivation fixtures
// and the crate is pinned to a specific version.
pub use kona_derive;

pub mod derivation;

pub mod execution;
