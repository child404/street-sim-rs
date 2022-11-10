//! This module creates API and algorithm of matching Candidates from file input.
//! Candidates in file should be separated by newline
#![allow(dead_code)]
use crate::candidate::Candidate;
use std::{
    cmp,
    fs::{self, File},
    io::{self, prelude::*, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use threadpool::ThreadPool;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) const PUNCTUATIONS: &[char] = &[
    '_', '\\', '(', ')', ',', '\"', '.', ';', ':', '\'', '-', '/', '+', '–', ' ',
];

type MatchFunc = fn(&str, &str) -> f64;

#[derive(Clone)]
pub enum MatchAlgo {
    Levenshtein,
    DamerauLevenshtein,
    JaroWinkler,
    Jaro,
    SorensenDice,
    Osa,
}

impl Default for MatchAlgo {
    fn default() -> Self {
        Self::Levenshtein
    }
}

impl MatchAlgo {
    pub fn get_func(&self) -> MatchFunc {
        match self {
            MatchAlgo::Levenshtein => strsim::normalized_levenshtein,
            MatchAlgo::Jaro => strsim::jaro,
            MatchAlgo::JaroWinkler => strsim::jaro_winkler,
            MatchAlgo::SorensenDice => strsim::sorensen_dice,
            MatchAlgo::DamerauLevenshtein => strsim::normalized_damerau_levenshtein,
            MatchAlgo::Osa => |a, b| {
                1.0 - (strsim::osa_distance(a, b) as f64)
                    / (a.chars().count().max(b.chars().count()) as f64)
            },
        }
    }
}

pub struct Sensitivity {
    pub value: f64,
}

impl Sensitivity {
    pub fn new(sensitivity: f64) -> Self {
        if 1.0 - sensitivity < 0.0 {
            panic!(
                "Sensitivity should be lower or equal than 1.0, but the value was {}",
                sensitivity
            );
        }
        if sensitivity - 1e-10 < 0.0 {
            panic!(
                "Sensitivity should be larger or equal than 0.0, but the value was {}",
                sensitivity
            );
        }
        Self { value: sensitivity }
    }
}

pub struct TextMatcher {
    pub sensitivity: Sensitivity,
    pub num_to_keep: usize,
    pub match_func: MatchFunc,
}

impl TextMatcher {
    /// `sensitivity` - the lower threshold of the `similarity` value that still should be kept
    ///
    /// `num_to_keep` - the number of candidates to keep after the matching process
    ///
    /// # Panics
    /// Panics if the sensitivity value is lower than 0.0 or larger than 1.0
    pub fn new(sensitivity: f64, num_to_keep: usize, match_algo: MatchAlgo) -> Self {
        // TODO: add Sensitivity as param instead of f64
        Self {
            sensitivity: Sensitivity::new(sensitivity),
            num_to_keep,
            match_func: match_algo.get_func(),
        }
    }

    pub fn find_matches_from_str(&self, candidates: &[String], text: &str) -> Vec<Candidate> {
        let mut candidates = candidates
            .iter()
            .flat_map(|candidate| {
                let similarity = (self.match_func)(text, &candidate.replace(PUNCTUATIONS, ""));
                if similarity - self.sensitivity.value > 0.0 {
                    Some(Candidate {
                        text: candidate.to_owned(),
                        similarity,
                        file_found: PathBuf::new(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<Candidate>>();
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        candidates[..cmp::min(self.num_to_keep, candidates.len())].to_vec()
    }

    pub fn find_matches_from(&self, candidates: &[Candidate], text: &str) -> Vec<Candidate> {
        let mut candidates = candidates
            .iter()
            .flat_map(|candidate| {
                let similarity = (self.match_func)(text, &candidate.text.replace(PUNCTUATIONS, ""));
                if similarity - self.sensitivity.value > 0.0 {
                    Some(Candidate {
                        similarity,
                        ..candidate.clone()
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<Candidate>>();
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        candidates[..cmp::min(self.num_to_keep, candidates.len())].to_vec()
    }

    pub fn cfind_matches_in_file(
        sens: f64,
        num_to_keep: usize,
        text: &str,
        file: &Path,
        num_of_threads: Option<usize>,
        match_algo: MatchAlgo,
    ) -> io::Result<Vec<Candidate>> {
        let lines = BufReader::new(File::open(file)?)
            .lines()
            .flatten()
            .collect::<Vec<String>>();
        let num_of_threads =
            num_of_threads.unwrap_or_else(|| thread::available_parallelism().unwrap().get());

        let candidates = Arc::new(Mutex::new(Vec::with_capacity(num_of_threads * num_to_keep)));
        let sensitivity = Sensitivity::new(sens);
        let match_func = match_algo.get_func();

        let handles = lines
            .chunks(lines.len() / num_of_threads + 1)
            .map(|chunk| {
                let candidates = candidates.clone();
                let chunk = chunk.to_vec();
                let text = text.to_string();
                let file = file.to_path_buf();
                thread::spawn(move || {
                    let mut matches = Vec::with_capacity(chunk.len());
                    for candidate in chunk {
                        let similarity = (match_func)(&text, &candidate.replace(PUNCTUATIONS, ""));
                        if similarity - sensitivity.value > 0.0 {
                            matches.push(Candidate {
                                text: candidate.to_string(),
                                similarity,
                                file_found: file.clone(),
                            });
                        }
                    }
                    matches.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
                    for candidate in &matches[..cmp::min(num_to_keep, matches.len())] {
                        candidates.lock().unwrap().push(candidate.clone());
                    }
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().expect("undefined error with threads!");
        }
        let mut candidates = candidates.lock().unwrap().to_vec();
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        Ok(candidates[..cmp::min(num_to_keep, candidates.len())].to_vec())
    }

    /// Search through file for candidates each on new line
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_matcher_rs::{TextMatcher, SearchAlgo};
    /// # use std::path::PathBuf;
    /// #
    /// # fn main() {
    /// #     let mat = TextMatcher::new(0.8, 1, SearchAlgo::default()).find_matches_in_file("qu du seujet 36", &PathBuf::from("./test_data/plzs/1201"), None).unwrap();
    /// #     assert_eq!(mat[0].text, "quai du seujet 36".to_string())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If this function encounteres any problem with reading the file, an error variant will be returned
    pub fn find_matches_in_file(
        &self,
        text: &str,
        file: &Path,
        is_first_let_eq: Option<bool>,
    ) -> io::Result<Vec<Candidate>> {
        // TODO: make it concurrent
        let mut candidates = Vec::new(); // try to use .clear() here with &mut TextMatcher
        let reader = BufReader::new(File::open(file)?);
        let is_first_let_eq = is_first_let_eq.unwrap_or(false);
        for candidate_txt in reader.lines().flatten() {
            if is_first_let_eq
                && text.graphemes(true).next().unwrap()
                    != candidate_txt.graphemes(true).next().unwrap()
            {
                continue;
            }
            // TODO: think on removing punctuations while comparing strings, i.e.: candidate_text.replace(PUNCTUATIONS, "").replace('/', "")
            let similarity = (self.match_func)(text, &candidate_txt.replace(PUNCTUATIONS, ""));
            if similarity - self.sensitivity.value > 0.0 {
                candidates.push(Candidate {
                    text: candidate_txt,
                    similarity,
                    file_found: file.to_path_buf(),
                })
            }
        }
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        Ok(candidates[..cmp::min(self.num_to_keep, candidates.len())].to_vec())
    }

    /// Search through files in directory for candidates each on new line
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use text_matcher_rs::{TextMatcher, SearchAlgo};
    /// # use std::path::PathBuf;
    /// #
    /// # fn main() {
    /// #     let mat = TextMatcher::find_matches_in_dir(0.8, 1, "qu du seujet 36", &PathBuf::from("./test_data/plzs/"), None, None, SearchAlgo::default());
    /// #     assert_eq!(mat[0].text, "quai du seujet 36".to_string())
    /// # }
    /// ```
    pub fn find_matches_in_dir(
        sensitivity: f64,
        num_to_keep: usize,
        text: &str,
        path_to_dir: &Path,
        num_of_threads: Option<usize>,
        is_first_let_eq: Option<bool>,
        search_method: MatchAlgo,
    ) -> Vec<Candidate> {
        let matches: Arc<Mutex<Vec<Candidate>>> = Arc::new(Mutex::new(Vec::new()));
        let pool = ThreadPool::new(
            num_of_threads.unwrap_or_else(|| thread::available_parallelism().unwrap().get()),
        );

        for file in fs::read_dir(path_to_dir)
            .expect("Directory exists")
            .flatten()
        {
            let text = text.to_string();
            let matches = matches.clone();
            let matcher = TextMatcher::new(sensitivity, num_to_keep, search_method.clone());
            pool.execute(move || {
                if let Ok(candidates) =
                    matcher.find_matches_in_file(&text, &file.path(), is_first_let_eq)
                {
                    for candidate in candidates {
                        matches.lock().unwrap().push(candidate);
                    }
                }
            });
        }
        pool.join();

        let mut matches = matches.lock().unwrap().to_vec();
        matches.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        matches[..cmp::min(num_to_keep, matches.len())].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const DATA_FILE: &str = "./test_data/plzs/1201";
    const DATA_DIR: &str = "./test_data/plzs/";

    #[test]
    #[should_panic(expected = "should be larger or equal than")]
    fn sensitivity_lower_than_zero() {
        Sensitivity::new(-1.0);
    }

    #[test]
    #[should_panic(expected = "should be lower or equal than")]
    fn sensitivity_larger_than_one() {
        Sensitivity::new(1.1);
    }

    #[test]
    fn high_sensitivity() {
        let matches = TextMatcher::new(0.99, 5, MatchAlgo::default())
            .find_matches_in_file("qu du seujet 36", &PathBuf::from(DATA_FILE), Some(true))
            .unwrap();
        assert_eq!(
            matches.len(),
            0,
            "Expected empty Vec, but value was {:?}",
            matches
        );
    }

    #[test]
    fn zero_to_keep() {
        let matches = TextMatcher::new(0.7, 0, MatchAlgo::default())
            .find_matches_in_file("qu du seujet 36", &PathBuf::from(DATA_FILE), Some(true))
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    fn assert_candidate(expected: &str, actual: &Candidate) {
        let sim_lower_threshold = 0.7;
        assert_eq!(expected.to_string(), actual.text);
        assert!(
            actual.similarity > sim_lower_threshold,
            "Similarity expected > {}, but value was {}",
            sim_lower_threshold,
            actual.similarity
        );
    }

    #[test]
    fn nomal_sensitivity() {
        let best_match = &TextMatcher::new(0.7, 5, MatchAlgo::default())
            .find_matches_in_file("qu du seujet 36", &PathBuf::from(DATA_FILE), Some(true))
            .unwrap()[0];
        assert_candidate("quai du seujet 36", best_match);
    }

    #[test]
    #[ignore]
    fn find_in_dir() {
        let best_match = &TextMatcher::find_matches_in_dir(
            0.1,
            5,
            "qu du seujet 36",
            &PathBuf::from(DATA_DIR),
            Some(4),
            Some(true),
            MatchAlgo::default(),
        )[0];
        assert_candidate("quai du seujet 36", best_match);
    }
}
