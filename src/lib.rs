//! 
//! Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
//! characters of a search string and calculates a score based on matching
//! consecutive/close groups of characters.
//! 
//! Walks _all_ paths through the string that is being searched.
//! 
//! # Usage
//! 
//! Basic usage:
//! 
//! ```rust
//! use sublime_fuzzy::best_match;
//! 
//! let s = "some search thing";
//! let search = "something";
//! let result = best_match(search, s).unwrap();
//! 
//! // Output: score: 368
//! println!("score: {:?}", result.score());
//! ```
//! 
//! `Match.continuous_matches()` returns a list of consecutive matches
//! (`(start_index, length)`). Based on those the input string can be formatted.
//! `sublime_fuzzy` provides a simple formatting function that wraps matches in
//! tags.
//! 
//! ```rust
//! use sublime_fuzzy::{best_match, format_simple};
//! 
//! let s = "some search thing";
//! let search = "something";
//! let result = best_match(search, s).unwrap();
//! 
//! // Output: <span>some</span> search <span>thing</span>
//! println!("formatted: {:?}", format_simple(&result, s, "<span>", "</span>"));
//! ```
//! 
//! Adjust scoring:
//! 
//! ```rust
//! use sublime_fuzzy::{FuzzySearch, ScoreConfig};
//! 
//! let mut search = FuzzySearch::new("something", "some search thing");
//! 
//! let config = ScoreConfig {
//!     bonus_consecutive: 20,
//!     penalty_distance: 8
//! };
//! // Weight consecutive matching chars less.
//! search.set_score_config(config);
//! 
//! println!("result: {:?}", search.best_match());
//! ```
//! 
//! **Note:** This module removes any whitespace in the pattern (`'something'`
//! in the examples above). It does not apply any other formatting. Lowercasing
//! the inputs for example has to be done manually.
//! 

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

    /// Creates an instance with capacity of the matches vector set to
    /// `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Match {
            score: 0,
            matches: Vec::with_capacity(capacity),
        }
    }

    /// Returns the score of this match.
    pub fn score(&self) -> isize {
        self.score
    }

    /// Recalculates the score, stores it in the `Match` and then returns it.
    pub fn calc_score(&mut self, config: &ScoreConfig) -> isize {
        self.score = calc_score(&self.matches, config);
        self.score
    }

    /// Returns the list of matched char positions.
    pub fn matches(&self) -> &Vec<usize> {
        &self.matches
    }

    /// Groups the individual char matches into continuous match chains.
    /// Returns a list of `(start_index, length)` pairs.
    pub fn continuous_matches(&self) -> Vec<(usize, usize)> {
        let mut groups = Vec::new();

        let mut current_start = 0;
        let mut current_len = 0;

        let mut last_index = 0;
        let mut is_first_index = true;

        for index in &self.matches {
            if !is_first_index && index - 1 == last_index {
                current_len += 1;
            } else {
                if current_len > 0 {
                    groups.push((current_start, current_len));
                }
                current_start = index.clone();
                current_len = 1;

                is_first_index = false;
            }
            last_index = index.clone();
        }

        if current_len > 0 {
            groups.push((current_start, current_len));
        }

        groups
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

pub struct ScoreConfig {
    pub bonus_consecutive: isize,
    pub penalty_distance: isize,
}

/// Container for search configuration.
/// Allows for adjusting the factors used to calculate the match score.
///
/// # Examples
/// 
/// Basic usage:
///
///     use sublime_fuzzy::{FuzzySearch, ScoreConfig};
///
///     let mut search = FuzzySearch::new("something", "some search thing");
///
///     let config = ScoreConfig {
///         bonus_consecutive: 20,
///         penalty_distance: 8
///     };
///     // Weight consecutive matching chars less.
///     search.set_score_config(config);
///
///     println!("result: {:?}", search.best_match());
///
pub struct FuzzySearch<'a> {
    score_config: ScoreConfig,
    pattern: &'a str,
    charmap: CharMap,
    best_match: Match,
    index_stack: Vec<usize>,
    score_stack: Vec<isize>,
    consec_len_stack: Vec<usize>,
}

impl<'a> FuzzySearch<'a> {
    /// Creates a default `FuzzySearch` instance.
    pub fn new(pattern: &'a str, string: &'a str) -> Self {
        if pattern.len() == 0 || string.len() == 0 {
            panic!("Inputs can't be empty!");
        }

        FuzzySearch {
            score_config: ScoreConfig {
                bonus_consecutive: 16,
                penalty_distance: 4,
            },
            pattern: pattern,
            charmap: build_charmap(string),
            best_match: Match::with_capacity(pattern.len()),
            index_stack: Vec::with_capacity(pattern.len()),
            score_stack: Vec::with_capacity(pattern.len()),
            consec_len_stack: Vec::with_capacity(pattern.len()),
        }
    }

    /// Sets score used to adjust for distance between matching chars.
    pub fn set_score_config(&mut self, config: ScoreConfig) {
        self.score_config = config;
    }

    /// Gets the best match for the given search string.
    pub fn best_match(&mut self) -> Option<Match> {
        self.start_matching();

        Some(self.best_match.clone())
    }

    /// Starts the matching process.
    fn start_matching(&mut self) {
        let pattern_char = self.pattern.chars().nth(0).unwrap();

        if let Some(occurences) = occurences(pattern_char, &self.charmap, 0) {
            for o in occurences {
                self.match_char(1, o);
            }
        }
    }

    /// Matches char at `pattern_index` in `self.pattern`, offset search for
    /// further characters by `offset`.
    fn match_char(&mut self, pattern_index: usize, offset: usize) {
        // Calculate distance between this match and the last one.
        let dist = match self.index_stack.last() {
            Some(n) => offset - n,
            None => 0,
        };

        self.index_stack.push(offset);

        // Calculate the score for this step.
        let mut new_score = 0;

        match dist {
            // First match.
            0 => {
                self.consec_len_stack.push(0);
                self.score_stack.push(0);
            },
            // Consecutive match.
            1 => {
                let consec_len = self.consec_len_stack.last().unwrap() + 1;
                let consec_score = consec_len as isize * self.score_config.bonus_consecutive;
                self.consec_len_stack.push(consec_len);

                new_score = self.score_stack.last().unwrap() + consec_score;
                self.score_stack.push(new_score);
            },
            // Non-consecutive match.
            _ => {
                self.consec_len_stack.push(0);

                let penalty = (dist - 1) as isize * self.score_config.penalty_distance;
                new_score = self.score_stack.last().unwrap() - penalty;
                self.score_stack.push(new_score);
            }
        }

        // Calculate maximum possible score for the remaining chars in
        // `self.pattern`.
        let max_left_score = max_score(
            self.pattern.len() - (self.index_stack.len() - 1),
            self.consec_len_stack.last().unwrap().clone(),
            &self.score_config,
        );

        // Check if we can possibly reach a higher score than the current best.
        // If not, stop further matching for this path.
        if new_score + max_left_score < self.best_match.score() {
            self.pop_all();
            return
        }

        // Get the next char to look for.
        let pattern_char;

        if let Some(c) = self.pattern.chars().nth(pattern_index) {
            pattern_char = c;
        } else {
            self.score_current();
            self.pop_all();
            return
        }

        if let Some(occurences) = occurences(pattern_char, &self.charmap, offset + 1) {
            for o in occurences {
                self.match_char(pattern_index + 1, o);
            }
        }
        self.pop_all();
    }

    /// Pops all stacks.
    fn pop_all(&mut self) {
        self.index_stack.pop();
        self.score_stack.pop();
        self.consec_len_stack.pop();
    }

    /// Replace current best match with the current match if it has a higher
    /// score.
    fn score_current(&mut self) {
        let current_score = self.score_stack.last().unwrap().clone();
        if current_score > self.best_match.score {
            let new_best = Match::with(current_score, self.index_stack.clone());
            self.best_match = new_best;
        }
    }
}

/// Calculates score for `positions`.
fn calc_score(positions: &Vec<usize>, config: &ScoreConfig) -> isize {
    let mut score: isize = 0;
    let mut last_pos: usize = 0;
    let mut is_first_pos = true;

    let mut consec_chain_len = 0;

    for pos in positions {
        // Ignore distance for first 
        if is_first_pos {
            last_pos = pos.clone();
            is_first_pos = false;
            continue;
        }

        let dist = pos - last_pos;

        if dist == 1 {
            consec_chain_len += 1;
            score += consec_chain_len * config.bonus_consecutive;
        } else {
            score -= dist as isize * config.penalty_distance;
            consec_chain_len = 0;
        }

        last_pos = pos.clone();
    }

    score
}

/// Calculate maximum possible score for a chain of length `num_positions` with
/// starting consecutive match count of `consec_chain_len`.
fn max_score(num_positions: usize, consec_chain_len: usize, config: &ScoreConfig) -> isize {
    // Consecutive score doesn't apply to first element, so subtract 1.
    let num_for_tri = num_positions - 1 + consec_chain_len;

    let triangular = num_for_tri * (num_for_tri + 1) / 2;

    // Assume we have the best possible combination, which is, currently, just
    // consecutive positions.
    let max_consecutive_score = triangular as isize * config.bonus_consecutive;
    // As they are consecutive the distance penalty is 0 and there are no other
    // scoring factors.
    max_consecutive_score
}

/// Gets all occurences of `what` in `target` starting from `search_offset`.
///
fn occurences(what: char, charmap: &CharMap, offset: usize) -> Option<Vec<usize>> {
    if let Some(occurences) = charmap.get(&what) {
        return Some(occurences.iter().filter(|&i| i >= &offset).map(|i| i.clone()).collect());
    }

    None
}

/// Maps all occurences of a character in `string` into a char => vec[indices]
/// dict.
fn build_charmap(string: &str) -> CharMap {
    let mut charmap = HashMap::new();

    for (i, c) in string.chars().enumerate() {
        charmap.entry(c).or_insert(Vec::new()).push(i);
    }

    charmap
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
pub fn best_match(pattern: &str, target: &str) -> Option<Match> {
    // Filter out whitespace, it's very unlikely someone matches for whitespace.
    // There is also a performance impact. Imagine a paragraph of text, there's
    // loads of whitespace in that. So this algorithm will branch off at every
    // space and calculate possibilites from there.
    // Benchmarks for long_start_close and long_middle_close:
    //      w spaces:   240,204,151 ns and 9,889,309 ns
    //      w/o spaces:  62,251,231 ns and   791,259 ns
    let condensed: String = pattern
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let mut searcher = FuzzySearch::new(&condensed, target);

    searcher.best_match()
}

/// Formats a `Match` by appending `before` before any matches and `after`
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
pub fn format_simple(result: &Match, string: &str, before: &str, after: &str) -> String {
    let str_before = before.to_owned();
    let str_after = after.to_owned();

    let mut pieces = Vec::new();

    let mut last_end = 0;

    for &(start, len) in &result.continuous_matches() {
        // Take piece between last match and this match.
        pieces.push(string.chars().skip(last_end).take(start - last_end).collect::<String>());
        // Add identifier for matches.
        pieces.push(str_before.clone());
        // Add actual match.
        pieces.push(string.chars().skip(start).take(len).collect());
        // Add after element.
        pieces.push(str_after.clone());
        last_end = start + len;
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
        println!("result: {:?}", best_match("obsion", "observablecollection"));
    }

    #[test]
    fn formatting_works_1() {
        use {best_match, format_simple};

        let s = "observablecollection";
        let search = "obscoll";

        let result = best_match(search, s).unwrap();

        assert_eq!(
            "<~obs~>ervable<~coll~>ection",
            format_simple(&result, s, "<~", "~>")
        );
    }

    #[test]
    fn formatting_works_2() {
        use {best_match, format_simple};
    
        let s = "some search thing";
        let search = "something";
        let result = best_match(search, s).unwrap();
         
        assert_eq!(
            "<b>some</b> search <b>thing</b>",
            format_simple(&result, s, "<b>", "</b>")
        );
    }

    #[test]
    fn formatting_works_3() {
        use {best_match, format_simple};
    
        let s = "The empty benchmark is there as a baseline. An anecdote:
            In my first compilation of the benchmark, I forgot to add -O to the
            rustc command line, and wound up with a few ns/iter on an empty
            benchmark. Thus, I now always have an empty benchmark in my list,
            to make sure I benchmark an optimized version.";
        let search = "emptybench";
        let result = best_match(search, s).unwrap();

        println!("{:?}", format_simple(&result, s, "<b>", "</b>"));
    }

    #[test]
    fn formatting_works_4() {
        use {best_match, format_simple};
    
        let s = "The empty benchmark is there as a baseline. An anecdote:
            In my first compilation of the benchmark, I forgot to add -O to the
            rustc command line, and wound up with a few ns/iter on an empty
            benchmark. Thus, I now always have an empty benchmark in my list,
            to make sure I benchmark an optimized version.";
        let search = "emptybaseline";
        let result = best_match(search, s).unwrap();

        println!("{:?}", format_simple(&result, s, "<b>", "</b>"));
    }

    #[test]
    fn occurences_work() {
        use {occurences, build_charmap};
    
        let charmap = build_charmap("some search thing");

        assert_eq!([3usize, 6usize].to_vec(), occurences('e', &charmap, 0).unwrap());
    }
}
