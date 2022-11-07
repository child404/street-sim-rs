//! This module creates API and algorithm of matching Candidates from file input.
//! Candidates in file should be separated by newline
use crate::candidate::Candidate;
use std::{
    cmp,
    fs::{self, File},
    io::{self, prelude::*, BufReader},
    path::Path,
    sync::{Arc, Mutex},
    thread,
};
use threadpool::ThreadPool;
use unicode_segmentation::UnicodeSegmentation;

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
}

impl TextMatcher {
    /// `sensitivity` - the lower threshold of the `similarity` value that still should be kept
    ///
    /// `num_to_keep` - the number of candidates to keep after the matching process
    ///
    /// # Panics
    /// Panics if the sensitivity value is lower than 0.0 or larger than 1.0
    pub fn new(sensitivity: f64, num_to_keep: usize) -> Self {
        Self {
            sensitivity: Sensitivity::new(sensitivity),
            num_to_keep,
        }
    }

    /// Search through file for candidates each on new line
    ///
    /// ```rust
    /// # use text_matcher_rs::TextMatcher;
    /// # use std::path::PathBuf;
    /// #
    /// # fn main() {
    /// #     let mat = TextMatcher::new(0.8, 1).find_matches_in_file("qu du seujet 36", &PathBuf::from("./test_data/plzs/1201"), None).unwrap();
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
            let similarity = strsim::normalized_levenshtein(text, &candidate_txt);
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
    /// ```rust
    /// # use text_matcher_rs::TextMatcher;
    /// # use std::path::PathBuf;
    /// #
    /// # fn main() {
    /// #     let mat = TextMatcher::find_matches_in_dir(0.8, 1, "qu du seujet 36", &PathBuf::from("./test_data/plzs/"), None, None);
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
            let matcher = TextMatcher::new(sensitivity, num_to_keep);
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
