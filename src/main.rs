mod candidate;
mod street_matcher;
mod text_matcher;
use crate::street_matcher::StreetMatcher;

fn main() {
    let text = "qu du seujet 36";
    let sm = StreetMatcher::new(text, None, None);
    println!("{:?}", sm.match_by_plz(None));
}
