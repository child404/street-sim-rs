//! This module creates an object that represents the candidate text to the target text
use std::{
    cmp::{Ordering, PartialEq},
    path::PathBuf,
};

#[derive(Debug, Clone, Default)]
pub struct Candidate {
    pub text: String,
    pub similarity: f64,
    pub file_found: PathBuf,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.file_found == other.file_found
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.similarity.partial_cmp(&other.similarity)
    }
}
