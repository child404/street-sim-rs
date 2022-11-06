mod candidate;
mod street_matcher;
mod text_matcher;

use crate::street_matcher::StreetMatcher;

fn main() {
    let text = "qu du seujet 36";
    let sm = StreetMatcher::default();
    assert_eq!(
        sm.match_by_plz(text, None).street.unwrap(),
        sm.match_by_place(text, None).street.unwrap()
    );
}
