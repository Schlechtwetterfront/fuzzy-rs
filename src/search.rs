use std::collections::HashMap;

use matching::Match;
use parsing::Occurrences;
use scoring::Scoring;

use crate::{
    parsing::{build_occurrences, process_query, Occurrence, QueryChar, QueryChars},
    scoring::DEFAULT_SCORING,
};

/// Describes a fuzzy search. Alternative to [`best_match`](crate::best_match) which allows for more configuration.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use sublime_fuzzy::{FuzzySearch, Scoring};
///
/// let scoring = Scoring::emphasize_word_starts();
///
/// let result = FuzzySearch::new("something", "Some Search Thing")
///     .score_with(&scoring)
///     .case_insensitive()
///     .best_match();
///
/// assert!(result.is_some());
/// ```
pub struct FuzzySearch<'a> {
    query: &'a str,
    target: &'a str,
    scoring: Option<&'a Scoring>,
    case_insensitive: bool,
}

impl<'a> FuzzySearch<'a> {
    /// Creates a new search to match `query` in `target`.
    ///
    /// Note that whitespace in query will be _ignored_.
    pub fn new(query: &'a str, target: &'a str) -> Self {
        FuzzySearch {
            query,
            target,
            scoring: None,
            case_insensitive: true,
        }
    }

    /// Use custom scoring values.
    ///
    /// If not specified will use `Scoring::default()`.
    pub fn score_with(mut self, scoring: &'a Scoring) -> Self {
        self.scoring = Some(scoring);

        self
    }

    /// Only match query chars in the target string if case matches.
    ///
    /// [`Scoring::bonus_match_case`] will not be applied if this is set (because a char match will
    /// always also be a case match).
    pub fn case_sensitive(mut self) -> Self {
        self.case_insensitive = false;

        self
    }

    /// Ignore case when matching query chars in the target string.
    ///
    /// If not only the char but also the case matches, [`Scoring::bonus_match_case`] will be added to
    /// the score. If that behavior is not wanted the bonus can be set to 0 with custom scoring.
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;

        self
    }

    /// Finds the best match of the query in the target string.
    ///
    /// Always tries to match the _full_ pattern. A partial match is considered
    /// invalid and will return [`None`]. Will also return [`None`] in case the query or
    /// target string are empty.
    pub fn best_match(self) -> Option<Match> {
        let processed_query = process_query(self.query);

        if processed_query.len() == 0 || self.target.len() == 0 {
            return None;
        }

        let occurrences = build_occurrences(&processed_query, self.target, self.case_insensitive);

        let searcher = FuzzySearcher::new(
            processed_query,
            self.scoring.unwrap_or(&DEFAULT_SCORING),
            self.case_insensitive,
        );

        searcher.best_match(&occurrences)
    }
}

struct FuzzySearcher<'a> {
    query: QueryChars,
    scoring: &'a Scoring,
    match_cache: HashMap<(usize, usize, usize), Option<Match>>,
    case_insensitive: bool,
}

impl<'a> FuzzySearcher<'a> {
    fn new(query: QueryChars, scoring: &'a Scoring, case_insensitive: bool) -> Self {
        FuzzySearcher {
            match_cache: HashMap::with_capacity(query.len() * query.len()),
            query,
            scoring,
            case_insensitive,
        }
    }

    #[inline(always)]
    fn queried_char(&self, qc: &QueryChar) -> char {
        if self.case_insensitive {
            qc.lower
        } else {
            qc.original
        }
    }

    #[inline(always)]
    fn case_bonus(&self, query_idx: usize, occurrence: &Occurrence) -> isize {
        if self.case_insensitive {
            self.query
                .get(query_idx)
                .map_or(0, |c| (c.original == occurrence.char) as isize)
                * self.scoring.bonus_match_case
        } else {
            0
        }
    }

    fn best_match(mut self, occurrences: &Occurrences) -> Option<Match> {
        let qc = self.query.get(0)?;

        occurrences
            .get(&self.queried_char(qc))?
            .iter()
            .filter_map(|o| self.match_(1, o, 0, &occurrences))
            .max()
    }

    fn match_(
        &mut self,
        query_idx: usize,
        occurrence: &Occurrence,
        consecutive: usize,
        occurrences: &Occurrences,
    ) -> Option<Match> {
        let this_key = (query_idx, occurrence.target_idx, consecutive);

        // Already scored sub-tree
        if let Some(cached) = self.match_cache.get(&this_key) {
            return cached.clone();
        }

        let next_char = self.query.get(query_idx);

        let score = consecutive as isize * self.scoring.bonus_consecutive
            + occurrence.is_start as isize * self.scoring.bonus_word_start
            + self.case_bonus(query_idx - 1, occurrence);

        let mut this_match = Match::with_matched(score, consecutive, vec![occurrence.target_idx]);

        // Successfully matched all query chars
        if next_char.is_none() {
            self.match_cache.insert(this_key, Some(this_match.clone()));

            return Some(this_match);
        }

        let occs = occurrences.get(&self.queried_char(next_char.unwrap()));

        // Reached end of target without matching all query chars
        if occs.is_none() {
            self.match_cache.insert(this_key, None);

            return None;
        }

        let best_match = occs
            .unwrap()
            .iter()
            .filter(|&o| o.target_idx > occurrence.target_idx)
            .filter_map(|o| {
                let distance = o.target_idx - occurrence.target_idx;

                let new_consecutive = if distance == 1 { consecutive + 1 } else { 0 };

                self.match_(query_idx + 1, o, new_consecutive, occurrences)
            })
            .max()
            .and_then(|m| {
                this_match.extend_with(&m, &self.scoring);

                Some(this_match)
            });

        self.match_cache.insert(this_key, best_match.clone());

        best_match
    }
}
