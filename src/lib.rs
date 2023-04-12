//! This crate is aimed to be a simple and fast solution for text-matching from the file with
//! more than 2 millions of lines, especially for streets in Switzerland.
//!
//! It serves as my first Rust project used for work and published out to the people
mod candidate;
pub mod mass;
pub mod text_sim;

pub use candidate::{Candidate, Sens, Text};
pub use mass::{Place, Plz, Street, StreetConfig};
pub use text_sim::{Config, SimAlgo};
