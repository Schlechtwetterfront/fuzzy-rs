//! 
//! Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
//! characters of a search string and calculates a score.
//! 
//! The score is based on several factors:
//! * **Word starts** like the `t` in `some_thing` get a bonus (`bonus_word_start`)
//! * **Consecutive matches** get an accumulative bonus for every consecutive match (`bonus_consecutive`)
//! * Matches with higher **coverage** (targets `some_release` (lower) versus `a_release` (higher) with pattern
//! `release`) will get a bonus multiplied by the coverage percentage (`bonus_coverage`)
//! * The **distance** between two matches will be multiplied with the `penalty_distance` penalty and subtracted from
//! the score
//!
//! The default bonus/penalty values are set to give a lot of weight to word starts. So a pattern `scc` will match
//! **S**occer**C**artoon**C**ontroller, not **S**o**cc**erCartoonController.
//! 
//! # Match Examples
//! 
//! With default weighting.
//! 
//! | Pattern       | Target string             | Result
//! | ---           | ---                       | ---
//! | `scc`         | `SoccerCartoonController` | **S**occer**C**artoon**C**ontroller
//! | `something`   | `some search thing`       | **some** search **thing**
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
//! println!("score: {:?}", result.score());
//! ```
//! 
//! `Match.continuous_matches()` returns a list of consecutive matches
//! (`(start_index, length)`). Based on those the input string can be formatted.
//!
//! `sublime_fuzzy` provides a simple formatting function that wraps matches in
//! tags:
//! 
//! ```rust
//! use sublime_fuzzy::{best_match, format_simple};
//! 
//! let s = "some search thing";
//! let search = "something";
//! let result = best_match(search, s).unwrap();
//! 
//! assert_eq!(
//!     format_simple(&result, s, "<span>", "</span>"),
//!     "<span>some</span> search <span>thing</span>"
//! );
//! ```
//! 
//! The weighting of the different factors can be adjusted:
//! 
//! ```rust
//! use sublime_fuzzy::{FuzzySearch, ScoreConfig};
//!
//! let case_insensitive = true;
//! 
//! let mut search = FuzzySearch::new("something", "some search thing", case_insensitive);
//! 
//! let config = ScoreConfig {
//!     bonus_consecutive: 12,
//!     bonus_word_start: 64,
//!     bonus_coverage: 64,
//!     penalty_distance: 4,
//! };
//!
//! search.set_score_config(config);
//! 
//! println!("result: {:?}", search.best_match());
//! ```
//! 
//! **Note:** Any whitespace in the pattern (`'something'`
//! in the examples above) will be removed.
//! 

use std::collections::{HashMap};

mod matching;
pub use matching::{Match};

mod scoring;
pub use scoring::{ScoreConfig};
use scoring::Score;

mod parse;

/// Container for search configuration.
/// Allows for adjusting the factors used to calculate the match score.
///
/// # Examples
/// 
/// Basic usage:
///
///     use sublime_fuzzy::{FuzzySearch, ScoreConfig};
///
///     let mut search = FuzzySearch::new("something", "some search thing", true);
///
///     let config = ScoreConfig {
///         bonus_consecutive: 20,
///         ..Default::default()
///     };
///     // Weight consecutive matching chars less.
///     search.set_score_config(config);
///
///     assert!(search.best_match().is_some());
///
pub struct FuzzySearch<'a> {
    score_config: ScoreConfig,

    /// The processed (possibly lowercased and whitespace removed) pattern
    pattern: String,
    /// The processed (possibly lowercased) target string
    target: &'a str,

    /// Map of char occurences in the target string
    charmap: parse::CharMap,
    /// Set of word start indices
    word_starts: parse::WordStartSet,

    /// Map of scores for subtrees. Key: `(pattern_index, search_offset, consecutive_matches)`
    ///
    /// * `pattern_index`: Index into the pattern (basically current char)
    /// * `search_offset`: Offset to start search for occurences of the next char from
    /// * `consecutive_matches`: Number of consecutive matches before this step.
    /// These are stored because the score for the subtree might differ if it's another consecutive match
    score_cache: HashMap<(usize, usize, usize), Option<Score>>,
}

impl<'a> FuzzySearch<'a> {
    /// Creates a default `FuzzySearch` instance.
    ///
    /// If `case_insensitive` is `true` both input strings will be lowercased
    /// (_after_ parsing the target string to find word starts).
    ///
    /// Note that any whitespace in `pattern` will be removed.
    pub fn new(pattern: &str, string: &'a str, case_insensitive: bool) -> Self {
        let (charmap, word_starts) = parse::process_target_string(string, case_insensitive);

        FuzzySearch {
            score_config: ScoreConfig { ..Default::default() },

            pattern: parse::condense(pattern, case_insensitive),
            target: string,

            charmap: charmap,
            word_starts: word_starts,

            score_cache: HashMap::new(),
        }
    }

    /// Sets score used to adjust for distance between matching chars.
    pub fn set_score_config(&mut self, config: ScoreConfig) {
        self.score_config = config;
    }

    /// Finds the highest-scoring match for the pattern in the target string.
    ///
    /// Returns `None` if the pattern couldn't be fully matched.
    pub fn best_match(&mut self) -> Option<Match> {
        if self.pattern.len() == 0 {
            return None;
        }

        let first_char = self.pattern.chars().nth(0).unwrap();

        let mut best_score: Option<Score> = None;

        if let Some(positions) = occurences(first_char, 0, &self.charmap) {
            best_score = positions.iter()
                // Get score for every position
                .map(|pos| self.score_deep(0, *pos, 0))
                // Filter out unfinished matches
                .filter_map(|s| s)
                // Only take the highest score
                .max_by_key(|t| t.score)
                .map_or(None, |s| Some(s));
        }

        let coverage_mult: f64 = self.pattern.len() as f64 / self.target.len() as f64;
        let coverage_score = self.score_config.bonus_coverage as f64 * coverage_mult;

        match best_score {
            None => None,
            Some(sc) => Some(Match::with(sc.score + coverage_score.round() as isize, sc.matches)),
        }
    }

    /// Recursively scores the "tree" of possible matches.
    ///
    /// Caches results for subtrees for faster calculation.
    fn score_deep(&mut self, pattern_idx: usize, offset: usize, consecutive: usize) -> Option<Score> {
        // Check if this "sub-tree" has already been scored
        if let Some(cached) = self.score_cache.get(&(pattern_idx, offset, consecutive)) {
            return cached.clone();
        }

        let next_index = pattern_idx + 1;

        let mut this_score = Score::new(
            scoring::consecutive_score(consecutive, &self.score_config),
            consecutive,
            Vec::new()
        );

        this_score.matches.push(offset);

        // Is a word start
        if self.word_starts.contains(&offset) {
            this_score.score += self.score_config.bonus_word_start;
        }

        // We have successfully matched the full pattern
        if next_index >= self.pattern.len() {
            self.score_cache.insert((pattern_idx, offset, consecutive), Some(this_score.clone()));
            return Some(this_score);
        }

        let next_char = self.pattern.chars().nth(next_index).unwrap();

        if let Some(occurences) = occurences(next_char, offset + 1, &self.charmap) {
            // Get the highest score of all sub-trees
            let best_score = occurences.iter()
                .map(|pos| {
                    if (pos - offset) == 1 {
                        self.score_deep(next_index, *pos, consecutive + 1)
                    } else {
                        self.score_deep(next_index, *pos, 0)
                    }
                })
                // Filter `None`s
                .filter_map(|s| s)
                // Take highest
                .max_by_key(|t| t.score)
                // Put `Score` into `Option<Score>`
                .map_or(None, |s| Some(s));

            match best_score {
                Some(best) => {
                    // Add best child score to current score
                    this_score.extend(best, &self.score_config);

                    self.score_cache.insert((pattern_idx, offset, consecutive), Some(this_score.clone()));

                    return Some(this_score);
                },
                None => {
                    self.score_cache.insert((pattern_idx, offset, consecutive), None);

                    return None;
                }
            }
        } else {
            self.score_cache.insert((pattern_idx, offset, consecutive), None);

            return None;
        }
    }
}

/// Gets all occurences of `what` in `target` starting from `search_offset`.
///
fn occurences(what: char, offset: usize, charmap: &parse::CharMap) -> Option<Vec<usize>> {
    if let Some(occurences) = charmap.get(&what) {
        return Some(occurences.iter().filter(|&i| i >= &offset).map(|i| i.clone()).collect());
    }

    None
}

/// Returns the best match for `pattern` in `target`.
///
/// Returns `None` if no match has been found (that includes "invalid" input like an empty target string).
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
    if parse::condense(pattern, true).len() == 0 || target.len() == 0 {
        return None;
    }

    let mut searcher = FuzzySearch::new(&pattern, target, true);

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
///     assert_eq!(
///         format_simple(&result, s, "<span>", "</span>"),
///         "<span>some</span> search <span>thing</span>"
///     );
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
    use {FuzzySearch, ScoreConfig, best_match};

    #[test]
    fn full_match() {
        assert!(best_match("test", "test").is_some());
    }

    #[test]
    fn any_match() {
        assert!(best_match("towers", "the two towers").is_some());
    }

    #[test]
    fn case_sensitive() {
        let mut search = FuzzySearch::new("ttt", "The Two Towers", false);
        assert!(search.best_match().is_none());
    }

    #[test]
    fn case_sensitive_2() {
        let mut search = FuzzySearch::new("TTT", "The Two Towers", false);
        assert!(search.best_match().is_some());
    }

    #[test]
    fn no_match_none() {
        assert_eq!(best_match("no", "yes"), None);
    }

    #[test]
    fn partial_match_none() {
        // Only full matches return Some(score).
        assert_eq!(best_match("partial", "part"), None);
    }

    #[test]
    fn distance_to_first_is_ignored() {
        // Set coverage bonus to zero, otherwise the shorter test target will
        // get more points.
        let score_cfg = ScoreConfig {
            bonus_coverage: 0,
            ..Default::default()
        };

        let mut search = FuzzySearch::new("release", "some_release", true);
        search.set_score_config(score_cfg.clone());

        let m1 = search.best_match().unwrap();

        let mut search2 = FuzzySearch::new("release", "a_release", true);
        search2.set_score_config(score_cfg);

        let m2 = search2.best_match().unwrap();

        assert_eq!(m1.score(), m2.score());
    }

    #[test]
    fn higher_coverage_scores_higher() {
        let m1 = best_match("release", "a_release").unwrap();
        let m2 = best_match("release", "some_release").unwrap();

        assert!(m1.score() > m2.score());
    }

    // Tests if word starts are selected over continuous matches.
    #[test]
    fn word_starts_score_higher() {
        let result = best_match("scc", "SccsCoolController").unwrap();
        let expected: Vec<usize> = vec![0, 4, 8];

        assert_eq!(result.matches(), &expected);
    }

    #[test]
    fn invalid_target() {
        assert_eq!(best_match("test", ""), None);
    }

    #[test]
    fn invalid_pattern() {
        assert_eq!(best_match("", "test"), None);
    }

    #[test]
    fn matches_1() {
        let expected: Vec<usize> = vec![0, 6, 13];
        let result = best_match("scc", "SoccerCartoonController").unwrap();

        assert_eq!(result.matches(), &expected);
    }

    #[test]
    fn matches_filename() {
        let expected: Vec<usize> = vec![13, 14, 15, 16];
        let result = best_match("path", "/some/folder/path.rs").unwrap();

        assert_eq!(result.matches(), &expected);
    }
}
