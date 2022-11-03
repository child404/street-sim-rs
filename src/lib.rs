mod candidate;
mod street_matcher;
mod text_matcher;

pub use candidate::Candidate;
pub use street_matcher::StreetMatcher;
pub use text_matcher::TextMatcher;

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::TextMatcher;
    use pretty_assertions::assert_eq;

    const DATA_FILE: &str = "./plzs/1201";
    const DATA_DIR: &str = "./data/";

    #[test]
    fn high_sensitivity() {
        let matcher = TextMatcher::new(0.99, 5, true);
        let matches = matcher
            .find_matches_in_file("qu du seujet 36", &PathBuf::from_str(DATA_FILE).unwrap())
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn zero_to_keep() {
        let matcher = TextMatcher::new(0.7, 0, true);
        let matches = matcher
            .find_matches_in_file("qu du seujet 36", &PathBuf::from_str(DATA_FILE).unwrap())
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn nomal_sensitivity() {
        let matcher = TextMatcher::new(0.7, 5, true);
        let matches = matcher
            .find_matches_in_file("qu du seujet 36", &PathBuf::from_str(DATA_FILE).unwrap())
            .unwrap();
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }

    #[test]
    fn multiple_files() {
        let matches = crate::text_matcher::TextMatcher::find_matches_in_dir(
            0.1,
            5,
            "qu du seujet 36",
            PathBuf::from_str(DATA_DIR).unwrap(),
            Some(4),
            true,
        );
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }
}
