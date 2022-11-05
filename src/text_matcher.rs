use crate::candidate::Candidate;
use std::{
    cmp,
    fs::{self, File},
    io::{self, prelude::*, BufReader},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use threadpool::ThreadPool;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextMatcher {
    sensitivity: f64,
    keep: usize,
}

impl TextMatcher {
    pub fn new(sensitivity: f64, keep: usize) -> Self {
        Self { sensitivity, keep }
    }

    /// Search through file for candidates each on new line
    ///
    /// ```rust
    /// # use text_matcher_rs::TextMatcher;
    /// #
    /// # fn main() {
    /// #     let mat = TextMatcher::new(0.8, 1).find_matches_in_file("qu du seujet 36", "./test_data/plzs/1201", None).unwrap();
    /// #     assert_eq!(mat[0].text, "quai du seujet 36".to_string())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If this function encounteres any problem with readiing the file, an error variant will be returned
    pub fn find_matches_in_file(
        &self,
        text: &str,
        file: &PathBuf,
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
            let similarity = strsim::normalized_levenshtein(text, &candidate_txt);
            if similarity - self.sensitivity > 0.0 {
                candidates.push(Candidate {
                    text: candidate_txt,
                    similarity,
                    file_found: file.to_path_buf(),
                })
            }
        }
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        Ok(candidates[..cmp::min(self.keep, candidates.len())].to_vec())
    }

    /// Search through files in directory for candidates each on new line
    ///
    /// ```rust
    /// # use text_matcher_rs::TextMatcher;
    /// #
    /// # fn main() {
    /// #     let mat = TextMatcher::find_matches_in_dir(0.8, 1, "qu du seujet 36", PathBuf::from("./test_data/plzs/"), None, None);
    /// #     assert_eq!(mat[0].text, "quai du seujet 36".to_string())
    /// # }
    /// ```
    pub fn find_matches_in_dir(
        sens: f64,
        keep: usize,
        text: &str,
        path_to_dir: PathBuf,
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
            let matcher = TextMatcher::new(sens, keep);
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
        matches[..cmp::min(keep, matches.len())].to_vec()
    }
}
