mod candidate;
mod text_matcher;
use crate::text_matcher::TextMatcher;
use std::{path::PathBuf, str::FromStr};

fn main() {
    // let tm = TextMatcher::new(0.8, 50);
    // let path_to_candidates = PathBuf::from_str("./test_data/plzs/1201").unwrap();
    // let text = "qu du seujet 36";
    // let matches_ = tm.find_matches(text, &path_to_candidates);
    // println!("{:?}", matches_);

    let path_to_candidates = PathBuf::from_str("./test_data/plzs/").unwrap();
    let text = "qu du seujet 36";
    let matches = TextMatcher::find_matches_in_dir(0.8, 50, text, path_to_candidates, Some(8));
    println!("{:?}", matches);
}
