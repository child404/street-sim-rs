//! This crate is aimed to be a simple and fast solution for text-matching from the file with
//! more than 2 millions of lines, especially for streets in Switzerland.
//!
//! Also, it serves as my first Rust project used for work and published out to the people
mod candidate;
mod street_matcher;
mod text_matcher;

pub use candidate::Candidate;
pub use street_matcher::{StreetMatcher, SwissStreet};
pub use text_matcher::{MatchAlgo, Sensitivity, TextMatcher};
