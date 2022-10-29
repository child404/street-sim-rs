mod candidate;
mod text_matcher;

pub use candidate::Candidate;
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
        let mut matcher = TextMatcher::new(0.99, 5);
        let matches = matcher
            .find_matches("qu du seujet 36", &PathBuf::from_str(DATA_FILE).unwrap())
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn zero_to_keep() {
        let mut matcher = TextMatcher::new(0.7, 0);
        let matches = matcher
            .find_matches("qu du seujet 36", &PathBuf::from_str(DATA_FILE).unwrap())
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn nomal_sensitivity() {
        let mut matcher = TextMatcher::new(0.7, 5);
        let matches = matcher
            .find_matches("qu du seujet 36", &PathBuf::from_str(DATA_FILE).unwrap())
            .unwrap();
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }

    #[test]
    fn multiple_files() {
        let matches = crate::text_matcher::find_matches_in_dir(
            0.1,
            5,
            "qu du seujet 36",
            PathBuf::from_str(DATA_DIR).unwrap(),
            Some(4),
        );
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }
}
