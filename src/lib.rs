mod candidate;
mod street_matcher;
mod text_matcher;

pub use candidate::Candidate;
pub use street_matcher::StreetMatcher;
pub use text_matcher::TextMatcher;

#[cfg(test)]
mod tests {
    use crate::{StreetMatcher, TextMatcher};
    use pretty_assertions::assert_eq;
    use std::{path::PathBuf, str::FromStr};

    const DATA_FILE: &str = "./test_data/plzs/1201";
    const DATA_DIR: &str = "./test_data/plzs/";

    #[test]
    fn street_matcher_with_plz() {
        let mat = StreetMatcher::new(None, None).match_by_plz("qu du seujet 36", Some(1201));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(
            mat.file_found.unwrap(),
            PathBuf::from("./test_data/plzs/1201")
        );
    }

    #[test]
    fn street_matcher_no_plz() {
        let mat = StreetMatcher::new(None, None).match_by_plz("qu du seujet 36", None);
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn street_matcher_wrong_plz() {
        let mat = StreetMatcher::new(None, None).match_by_plz("qu du seujet 36", Some(1231231));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn street_matcher_wrong_first_word() {
        let mat = StreetMatcher::new(None, None).match_by_plz("u du seujet 36", Some(1201));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(
            mat.file_found.unwrap(),
            PathBuf::from("./test_data/plzs/1201")
        )
    }

    #[test]
    fn street_matcher_wrong_first_word_no_plz() {
        let mat = StreetMatcher::new(None, None).match_by_plz("u du seujet 36", None);
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn street_matcher_wrong_first_word_wrong_plz() {
        let mat = StreetMatcher::new(None, None).match_by_plz("u du seujet 36", Some(2132131));
        assert_eq!(mat.street.unwrap(), String::from("quai du seujet 36"));
        assert_eq!(mat.file_found, None)
    }

    #[test]
    fn text_matcher_high_sensitivity() {
        let matches = TextMatcher::new(0.99, 5)
            .find_matches_in_file(
                "qu du seujet 36",
                &PathBuf::from_str(DATA_FILE).unwrap(),
                Some(true),
            )
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn text_matcher_zero_to_keep() {
        let matches = TextMatcher::new(0.7, 0)
            .find_matches_in_file(
                "qu du seujet 36",
                &PathBuf::from_str(DATA_FILE).unwrap(),
                Some(true),
            )
            .unwrap();
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn text_matcher_nomal_sensitivity() {
        let matcher = TextMatcher::new(0.7, 5);
        let matches = matcher
            .find_matches_in_file(
                "qu du seujet 36",
                &PathBuf::from_str(DATA_FILE).unwrap(),
                Some(true),
            )
            .unwrap();
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }

    #[test]
    fn text_matcher_multiple_files() {
        let matches = TextMatcher::find_matches_in_dir(
            0.1,
            5,
            "qu du seujet 36",
            PathBuf::from_str(DATA_DIR).unwrap(),
            Some(4),
            Some(true),
        );
        assert_eq!(matches[0].text, "quai du seujet 36".to_string());
        assert!(matches[0].similarity > 0.7);
    }
}
