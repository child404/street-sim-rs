//! This module creates an object that represents the candidate text to the target text
use std::{
    cmp,
    cmp::{Ordering, PartialEq},
    fmt, io,
};

pub(crate) const PUNCTUATIONS: &[char] = &[
    '_', '\\', '(', ')', ',', '\"', '.', ';', ':', '\'', '-', '/', '+', '–', ' ',
];
const SENS: f64 = 0.6;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    NotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("CandidatesNotFound"),
            Self::Io(err) => f.write_str(&err.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text {
    pub init: String,
    pub cleaned: String,
}

impl Text {
    pub fn new(text: &str) -> Self {
        Self {
            init: text.to_string(),
            cleaned: text.to_lowercase().replace(PUNCTUATIONS, ""),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub type SimResult = Result<Vec<Candidate>, Error>;

#[derive(Clone, Copy)]
pub struct Sens(pub f64);

impl Sens {
    pub fn new(sens: f64) -> Self {
        if 1.0 - sens < 0.0 {
            panic!(
                "sensitivity should be lower or equal than 1.0, but the value was {}",
                sens
            );
        }
        if sens - 1e-10 < 0.0 {
            panic!(
                "sensitivity should be larger or equal than 0.0, but the value was {}",
                sens
            );
        }
        Self(sens)
    }
}

impl Default for Sens {
    fn default() -> Self {
        Self(SENS)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Candidate {
    pub text: String,
    pub similarity: f64,
}

impl Candidate {
    pub fn from(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            similarity: 0.0,
        }
    }
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

pub struct Candidates;

impl Candidates {
    #[inline]
    pub fn from(candidates: &mut Vec<Candidate>, num_to_keep: usize) -> SimResult {
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        let values = candidates[..cmp::min(num_to_keep, candidates.len())].to_vec();
        if !values.is_empty() {
            Ok(values)
        } else {
            Err(Error::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "should be larger or equal than")]
    fn sensitivity_lower_than_zero() {
        Sens::new(-1.0);
    }

    #[test]
    #[should_panic(expected = "should be lower or equal than")]
    fn sensitivity_larger_than_one() {
        Sens::new(1.1);
    }
}
