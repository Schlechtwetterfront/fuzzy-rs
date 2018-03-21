//
// This contains the functions used to parse the target string.
//

pub type CharMap = HashMap<char, Vec<usize>>;
pub type WordStartSet = HashSet<usize>;

use std::collections::{HashSet, HashMap};

/// Removes whitespace and optionally lowercases.
pub fn condense(pattern: &str, case_insensitive: bool) -> String {
    if case_insensitive {
        pattern
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    } else {
        pattern
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }
}

/// Checks if a `char` is a word separator.
fn is_word_sep(c: char) -> bool {
    c.is_whitespace() || c == '_' || c == '/' || c == '\\' || c == '-' || c == '.' || c == ','
}

/// Maps all occurences of a character in `string` into a char => vec[indices]
/// dict and puts all word starts (i.e. `some_string` => `[0, 5]`) into a `Set`.
pub fn process_target_string(string: &str, ignore_case: bool) -> (CharMap, WordStartSet) {
    let mut charmap = HashMap::new();
    let mut word_starts = HashSet::new();

    let target_string = if ignore_case { string.to_lowercase().to_string() } else { string.to_owned() };

    let mut prev_is_upper = false;
    let mut prev_is_sep = true;
    let mut prev_is_start = false;

    for (i, (c, original)) in target_string.chars().zip(string.chars()).enumerate() {
        let mut is_start = false;
        let mut is_sep = is_word_sep(original);
        let mut is_upper = original.is_uppercase();

        charmap.entry(c).or_insert(Vec::new()).push(i);

        if is_sep {
            prev_is_upper = false;
            prev_is_sep = true;
            prev_is_start = false;

            continue;
        }

        if prev_is_sep {
            is_start = true;
        } else {
            if !prev_is_start && (prev_is_upper != is_upper) {
                is_start = true;
            }
        }

        if is_start {
            word_starts.insert(i);
        }

        prev_is_start = is_start;
        prev_is_sep = is_sep;
        prev_is_upper = is_upper;
    }

    (charmap, word_starts)
}
