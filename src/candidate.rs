use std::{
    cmp::{Ordering, PartialEq},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub text: String,
    pub similarity: f64,
    pub file_found: PathBuf,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.similarity.partial_cmp(&other.similarity)
    }
}
