//! This module creates API and algorithm of matching candidates from file input.
//! Candidates in file should be separated by newline
use crate::candidate::{self as cand, Candidate, Sens, SimResult, Text};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
    sync::{Arc, Mutex},
    thread,
};
use threadpool::ThreadPool;

#[derive(Clone)]
pub struct Config {
    sens: Sens,
    num_to_keep: usize,
    sim_func: SimFunc,
    num_of_threads: usize,
}

impl Config {
    /// `sensitivity` - the lower threshold of the `similarity` value that still should be kept
    ///
    /// `num_to_keep` - the number of candidates to keep after the matching process
    ///
    /// # Panics
    /// Panics if the sensitivity value is lower than 0.0 or larger than 1.0
    pub fn new(
        sens: Sens,
        num_to_keep: usize,
        algo: SimAlgo,
        num_of_threads: Option<usize>,
    ) -> Self {
        Self {
            sens,
            num_to_keep,
            sim_func: algo.into(),
            num_of_threads: num_of_threads
                .unwrap_or_else(|| thread::available_parallelism().unwrap().get()),
        }
    }
}

type SimFunc = fn(&str, &str) -> f64;

#[derive(Clone, Copy)]
pub enum SimAlgo {
    Levenshtein,
    DamerauLevenshtein,
    JaroWinkler, // same as Jaro, but gives more boost to texts with the same prefix
    Jaro,
    SorensenDice,
    Osa,
}

impl Default for SimAlgo {
    fn default() -> Self {
        Self::Levenshtein
    }
}

impl From<SimAlgo> for SimFunc {
    fn from(algo: SimAlgo) -> Self {
        match algo {
            SimAlgo::Levenshtein => strsim::normalized_levenshtein,
            SimAlgo::Jaro => strsim::jaro,
            SimAlgo::JaroWinkler => strsim::jaro_winkler,
            SimAlgo::SorensenDice => strsim::sorensen_dice,
            SimAlgo::DamerauLevenshtein => strsim::normalized_damerau_levenshtein,
            SimAlgo::Osa => |a, b| {
                1.0 - (strsim::osa_distance(a, b) as f64)
                    / (a.chars().count().max(b.chars().count()) as f64)
            },
        }
    }
}

#[inline]
fn cmp_texts(target: &Text, candidate: Text, config: &Config) -> Option<Candidate> {
    let similarity = (config.sim_func)(&target.cleaned, &candidate.cleaned);
    if similarity - config.sens.0 > 0.0 {
        Some(Candidate {
            text: candidate.init,
            similarity,
        })
    } else {
        None
    }
}

#[inline]
pub fn cmp_with_arr(candidates: &[String], text: &Text, cfg: &Config) -> SimResult {
    cand::try_sort_and_keep(
        &mut candidates
            .iter()
            .flat_map(|candidate| cmp_texts(text, Text::new(candidate.to_string()), cfg))
            .collect(),
        cfg.num_to_keep,
    )
}

#[inline]
pub fn fast_cmp_with_file(text: &Text, file: &Path, cfg: &Config) -> SimResult {
    let lines = BufReader::new(File::open(file)?)
        .lines()
        .flatten()
        .collect::<Vec<String>>();
    let pool = ThreadPool::new(cfg.num_of_threads);
    let matches = Arc::new(Mutex::new(Vec::with_capacity(
        cfg.num_of_threads * cfg.num_to_keep,
    )));
    for chunk in lines.chunks(lines.len() / cfg.num_of_threads + 1) {
        let candidates = matches.clone();
        let chunk = chunk.to_vec();
        let text = text.clone();
        let cfg = cfg.clone();
        pool.execute(move || {
            for candidate in cmp_with_arr(&chunk, &text, &cfg).unwrap_or_default() {
                candidates.lock().unwrap().push(candidate);
            }
        });
    }
    pool.join();
    let mut matches = matches.lock().unwrap();
    cand::try_sort_and_keep(&mut matches, cfg.num_to_keep)
}

/// Search through file for candidates each on new line
///
/// # Examples
///
/// ```rust
/// # use street_sim_rs::{text_sim, Sens, SimAlgo, Text, Config};
/// # use std::path::PathBuf;
/// #
/// # fn main() {
/// #     let cfg = Config::new(Sens::new(0.8), 1, SimAlgo::JaroWinkler, None);
/// #     let text = Text::new("qu du seujet 36");
/// #     let mat = text_sim::cmp_with_file(&text, &PathBuf::from("./test_data/streets_data/street_names.txt"), &cfg).unwrap();
/// #     assert_eq!(mat[0].text, "quai du seujet".to_string())
/// # }
/// ```
///
/// # Errors
///
/// If this function encounteres any problem with reading the file, an error variant will be returned
#[inline]
pub fn cmp_with_file(text: &Text, file: &Path, cfg: &Config) -> SimResult {
    cand::try_sort_and_keep(
        &mut BufReader::new(File::open(file)?)
            .lines()
            .flatten()
            .flat_map(|candidate| cmp_texts(text, Text::new(candidate), cfg))
            .collect(),
        cfg.num_to_keep,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const DATA_FILE: &str = "./test_data/streets_data/street_names.txt";

    #[test]
    fn find_in_file() {
        let cfg = Config::new(Sens::new(0.5), 1, SimAlgo::default(), None);
        let mat = cmp_with_file(
            &Text::new("qu du seujet 36".to_string()),
            &PathBuf::from(DATA_FILE),
            &cfg,
        )
        .unwrap();
        assert_eq!(Candidate::from("quai du seujet"), mat[0]);
    }

    #[test]
    fn fast_find_in_file() {
        let cfg = Config::new(Sens::new(0.5), 1, SimAlgo::default(), None);
        let matches = fast_cmp_with_file(
            &Text::new("qu du seujet 36".to_string()),
            &PathBuf::from(DATA_FILE),
            &cfg,
        )
        .unwrap();
        assert_eq!(Candidate::from("quai du seujet"), matches[0]);
    }

    #[test]
    fn find_from() {
        let cfg = Config::new(Sens::new(0.5), 1, SimAlgo::JaroWinkler, None);
        let matches = cmp_with_arr(
            &["foobar", "foa", "2foo", "abcd"]
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            &Text::new("foo".to_string()),
            &cfg,
        )
        .unwrap();
        assert_eq!(Candidate::from("2foo"), matches[0]);
    }
}
