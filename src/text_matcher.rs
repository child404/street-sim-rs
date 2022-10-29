// or rayon https://docs.rs/rayon/1.3.1/rayon/
// or next article: https://users.rust-lang.org/t/using-threads-or-async-faster-file-reading/45736
use crate::candidate::Candidate;
use std::{
    cmp,
    fs::{self, File},
    io::{self, prelude::*, BufReader},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use threadpool::ThreadPool;

pub struct TextMatcher {
    sensitivity: f64,
    keep: usize,
}

impl TextMatcher {
    pub fn new(sensitivity: f64, keep: usize) -> Self {
        Self { sensitivity, keep }
    }

    pub fn find_matches(&self, txt: &str, file: &PathBuf) -> io::Result<Vec<Candidate>> {
        let mut candidates = Vec::new(); // try to use .clear() here with &mut TextMatcher
        let reader = BufReader::new(File::open(file)?);
        for candidate_txt in reader.lines().flatten() {
            let sim = strsim::normalized_levenshtein(txt, &candidate_txt);
            if sim - self.sensitivity > 0.0 {
                candidates.push(Candidate {
                    text: candidate_txt,
                    similarity: sim,
                    file_found: file.to_path_buf(),
                })
            }
        }
        candidates.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
        Ok(candidates[..cmp::min(self.keep, candidates.len())].to_vec())
    }

    pub fn find_matches_in_dir(
        sens: f64,
        keep: usize,
        txt: &str,
        path_to_dir: PathBuf,
        num_of_threads: Option<usize>,
    ) -> Vec<Candidate> {
        let matches: Arc<Mutex<Vec<Candidate>>> = Arc::new(Mutex::new(Vec::new()));
        let pool = ThreadPool::new(num_of_threads.unwrap_or(2));

        for file in fs::read_dir(path_to_dir)
            .expect("Directory exists")
            .flatten()
        {
            let text = txt.to_string();
            let matches = matches.clone();
            pool.execute(move || {
                if let Ok(candidates) =
                    TextMatcher::new(sens, keep).find_matches(&text, &file.path())
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
