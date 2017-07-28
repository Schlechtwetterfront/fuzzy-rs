/*!

Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
characters of a search string and calculates a score based on matching
consecutive/close groups of characters. Tries to find the best match by scoring
multiple match paths.

# Usage

Basic usage:

```rust
use sublime_fuzzy::best_match;

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

// Output: score: 368
println!("score: {:?}", result.score());
```

Simple formatting:

```rust
use sublime_fuzzy::{best_match, format_simple};

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

// Output: <span>some</span> search <span>thing</span>
println!("formatted: {:?}", format_simple(&result, s, "<span>", "</span>"));
```

Adjust scoring:

```rust
use sublime_fuzzy::FuzzySearcher;

let mut search = FuzzySearcher::new();

search.set_search("something");
search.set_target("some search thing");

// Weight consecutive matching chars less.
search.set_score_consecutive(4);

println!("result: {:?}", search.best_match());
```

*/
use std::cmp::Ordering;
use std::collections::HashMap;

type CharMap = HashMap<char, Vec<usize>>;

/// A single search result.
/// Contains the calculated match score and all matches.
#[derive(Debug, Clone)]
pub struct Match {
    score: isize,
    matches: Vec<usize>,
}

impl Match {
    /// Creates an empty instance.
    pub fn new() -> Self {
        Match {
            score: 0,
            matches: Vec::new(),
        }
    }

    /// Creates an instance with the given score and matches.
    pub fn with(score: isize, matches: Vec<usize>) -> Self {
        Match {
            score: score,
            matches: matches,
        }        
    }

    pub fn score(&self) -> isize {
        self.score
    }

    pub fn matches(&self) -> &Vec<Match> {
        &self.matches
    }
}

impl Ord for Match {
    fn cmp(&self, other: &Match) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for Match {
    fn partial_cmp(&self, other: &Match) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Match {}

impl PartialEq for Match {
    fn eq(&self, other: &Match) -> bool {
        self.score == other.score
    }
}

/// Container for search configuration.
/// Allows for adjusting the factors used to calculate the match score.
///
/// # Examples
/// 
/// Basic usage:
///
///     use sublime_fuzzy::FuzzySearcher;
///
///     let mut search = FuzzySearcher::new();
///
///     search.set_search("something");
///     search.set_target("some search thing");
///
///     // Weight consecutive matching chars less.
///     search.set_score_consecutive(4);
///
///     println!("result: {:?}", search.best_match());
///
pub struct FuzzySearcher {
    score_distance: usize,
    score_found_char: usize,
    score_consecutive: usize,
    target: String,
    search: String,
}

impl FuzzySearcher {
    /// Creates a default `FuzzySearcher` instance.
    pub fn new() -> Self {
        FuzzySearcher {
            score_distance: 4,
            score_found_char: 16,
            score_consecutive: 16,
            target: String::new(),
            search: String::new(),
        }
    }

    /// Sets score used to adjust for distance between matching chars.
    pub fn set_score_distance(&mut self, score: usize) {
        self.score_distance = score;
    }

    /// Sets score added for each found char.
    pub fn set_score_found_char(&mut self, score: usize) {
        self.score_found_char = score;
    }

    /// Sets score added for every consecutive matching char.
    pub fn set_score_consecutive(&mut self, score: usize) {
        self.score_consecutive = score;
    }

    /// Sets search string.
    pub fn set_target(&mut self, target: &str) {
        self.target = target.to_owned();
    }

    /// Sets string to be searched in.
    pub fn set_search(&mut self, search: &str) {
        self.search = search.to_owned();
    }

    /// Calculates score for a match chain and accumulates it all in a 
    /// `SearchResult`.
    pub fn chain_score(&mut self, match_chain: &Vec<usize>) -> SearchResult {
        let mut matches = Vec::new();
        let mut score: isize = 0;

        let mut consecutive_char_score = 0;

        let mut last_index = 0;

        let mut current_match = Match::new();

        let mut first_char = true;

        for pos in match_chain {
            if *pos == last_index + 1 && !first_char {
                consecutive_char_score += self.score_consecutive;
                current_match.len += 1;
            } else {
                if current_match.len > 0 {
                    matches.push(current_match);
                }
                consecutive_char_score = 0;
                current_match = Match::with(*pos, 1);
            }

            let mut dist: isize = 0;
            if !first_char {
                dist = max(*pos as isize - last_index as isize - 1, 0) as isize;
            }

            first_char = false;

            score -= dist * self.score_distance as isize;
            score += self.score_found_char as isize;
            score += consecutive_char_score as isize;

            last_index = *pos;
        }

        if current_match.len > 0 {
            matches.push(current_match);
        }

        SearchResult::with(score, matches)
    }

    /// Gets the best match for the given search string.
    pub fn best_match(&mut self) -> Option<SearchResult> {
        let chains = match_chains(&self.search, &self.target, 0, 0, &mut Vec::new());
        let mut results: Vec<SearchResult> = chains.iter().map(|x| self.chain_score(x)).collect();
        results.sort();
        results.reverse();

        if let Some(r) = results.first() {
            return Some(r.to_owned());
        }
        None
    }
}

/// Gets all occurences of `what` in `target` starting from `search_offset`.
///
fn occurences(what: char, target: &str, search_offset: usize) -> Option<Vec<usize>> {
    let mut occurences = Vec::new();
    let mut start_index = search_offset;
    loop {
        if let Some(next_start) = target.chars().skip(start_index).position(|x| x == what) {
            occurences.push(next_start + start_index);
            start_index = next_start + start_index + 1;
        } else {
            break;
        }
    }
    if occurences.len() > 0 {
        Some(occurences)
    } else {
        None
    }
}

/// Gets all possible match chains of `search` in `target`.
///
fn match_chains(search: &str, target: &str, search_offset: usize, mut index: usize, list: &Vec<usize>) -> Vec<Vec<usize>> {
    let mut search_char;

    if let Some(c) = search.chars().nth(index) {
        search_char = c;
    } else {
        let mut container = Vec::new();
        container.push(list.clone());
        return container;
    }

    let occurences = loop {
        if let Some(result) = occurences(search_char, target, search_offset) {
            break result;
        } else {
            index += 1;
            if let Some(new_c) = search.chars().nth(index) {
                search_char = new_c;
            } else {
                let mut container = Vec::new();
                container.push(list.clone());
                return container;
            }
        }
    };

    let mut results: Vec<Vec<usize>> = Vec::new();
    for o in occurences {
        let mut list_cpy = list.clone();
        list_cpy.push(o);

        results.append(&mut match_chains(search, target, o + 1, index + 1, &mut list_cpy));
    }

    results
}

/// Returns the best match for `search` in `target`.
///
/// # Examples
///
/// Basic usage:
///
///     use sublime_fuzzy::best_match;
///
///     let s = "some search thing";
///     let search = "something";
///     let result = best_match(search, s).unwrap();
///
///     // Output: score: 368
///     println!("score: {:?}", result.score());
///
pub fn best_match(search: &str, target: &str) -> Option<SearchResult> {
    let mut searcher = FuzzySearcher::new();
    searcher.set_search(search);
    searcher.set_target(target);

    searcher.best_match()
}

/// Formats a `SearchResult` by appending `before` before any matches and `after`
/// after any matches.
///
/// # Examples
///
/// Basic usage:
/// 
///     use sublime_fuzzy::{best_match, format_simple};
///
///     let s = "some search thing";
///     let search = "something";
///     let result = best_match(search, s).unwrap();
///     
///     // Output: <span>some</span> search <span>thing</span>
///     println!("formatted: {:?}", format_simple(&result, s, "<span>", "</span>"));
///
pub fn format_simple(result: &SearchResult, string: &str, before: &str, after: &str) -> String {
    let str_before = before.to_owned();
    let str_after = after.to_owned();

    let mut pieces = Vec::new();

    let mut last_end = 0;

    for m in &result.matches {
        // Take piece between last match and this match.
        pieces.push(string.chars().skip(last_end).take(m.start - last_end).collect::<String>());
        // Add identifier for matches.
        pieces.push(str_before.clone());
        // Add actual match.
        pieces.push(string.chars().skip(m.start).take(m.len).collect());
        // Add after element.
        pieces.push(str_after.clone());
        last_end = m.start + m.len;
    }

    // If there's characters left after the last match, make sure to append them.
    if last_end != string.len() {
        pieces.push(string.chars().skip(last_end).take_while(|_| true).collect::<String>());
    }
    return pieces.join("");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use best_match;
        println!("result: {:?}", best_match("obsinline", "observablecollection"));
    }
    #[test]
    fn formatting_works() {
        use {best_match, format_simple};
        let s = "observablecollection";
        println!(
            "formatted: {:?}",
            format_simple(
                &best_match("obsinline", s).unwrap(),
                s,
                "<span>",
                "</span>"
            )
        );
    }
    #[test]
    fn formatting_works_2() {
        use {best_match, format_simple};
    
        let s = "some search thing";
        let search = "something";
        let result = best_match(search, s).unwrap();
         
        println!("formatted: {:?}", format_simple(&result, s, "<b>", "</b>"));
    }
}
