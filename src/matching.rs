use std::{cmp::Ordering, slice::Iter};

use crate::Scoring;

/// A (possible partial) match of query within the target string. Matched chars
/// are stored as indices into the target string.
///
/// The score is not clamped to any range and can be negative.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Match {
    /// Accumulative score
    score: isize,
    /// Count of current consecutive matched chars
    consecutive: usize,
    /// Matched char indices
    matched: Vec<usize>,
}

impl Match {
    /// Creates a new match with the given scoring and matched indices.
    pub(crate) fn with_matched(score: isize, consecutive: usize, matched: Vec<usize>) -> Self {
        Match {
            score,
            consecutive,
            matched,
        }
    }

    /// Returns the accumulative score for this match.
    pub fn score(&self) -> isize {
        self.score
    }

    /// Returns an iterator over the matched char indices.
    pub fn matched_indices(&self) -> Iter<usize> {
        self.matched.iter()
    }

    /// Returns an iterator that groups the individual char matches into groups.
    pub fn continuous_matches(&self) -> ContinuousMatches {
        ContinuousMatches {
            matched: &self.matched,
            current: 0,
        }
    }

    /// Extends this match with `other`.
    pub fn extend_with(&mut self, other: &Match, scoring: &Scoring) {
        self.score += other.score;
        self.consecutive += other.consecutive;

        if let (Some(last), Some(first)) = (self.matched.last(), other.matched.first()) {
            let distance = first - last;

            match distance {
                0 => {}
                1 => {
                    self.consecutive += 1;
                    self.score += self.consecutive as isize * scoring.bonus_consecutive;
                }
                _ => {
                    self.consecutive = 0;
                    let penalty = (distance as isize - 1) * scoring.penalty_distance;
                    self.score -= penalty;
                }
            }
        }

        self.matched.extend(&other.matched);
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

/// Describes a continuous group of char indices
#[derive(Debug)]
pub struct ContinuousMatch {
    start: usize,
    len: usize,
}

impl ContinuousMatch {
    pub(crate) fn new(start: usize, len: usize) -> Self {
        ContinuousMatch { start, len }
    }

    /// Returns the start index of this group.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the length of this group.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Eq for ContinuousMatch {}

impl PartialEq for ContinuousMatch {
    fn eq(&self, other: &ContinuousMatch) -> bool {
        self.start == other.start && self.len == other.len
    }
}

/// Iterator returning [`ContinuousMatch`]es from the matched char indices in a [`Match`]
pub struct ContinuousMatches<'a> {
    matched: &'a Vec<usize>,
    current: usize,
}

impl<'a> Iterator for ContinuousMatches<'_> {
    type Item = ContinuousMatch;

    fn next(&mut self) -> Option<ContinuousMatch> {
        let mut start = None;
        let mut len = 0;

        let mut last_idx = None;

        for idx in self.matched.iter().cloned().skip(self.current) {
            start = start.or(Some(idx));

            if last_idx.is_some() && (idx - last_idx.unwrap() != 1) {
                return Some(ContinuousMatch::new(start.unwrap(), len));
            }

            self.current += 1;
            len += 1;
            last_idx = Some(idx);
        }

        if last_idx.is_some() {
            return Some(ContinuousMatch::new(start.unwrap(), len));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::Scoring;

    use super::{ContinuousMatch, Match};

    #[test]
    fn continuous() {
        let m = Match::with_matched(0, 0, vec![0, 1, 2, 5, 6, 10]);

        assert_eq!(
            m.continuous_matches().collect::<Vec<ContinuousMatch>>(),
            vec![
                ContinuousMatch { start: 0, len: 3 },
                ContinuousMatch { start: 5, len: 2 },
                ContinuousMatch { start: 10, len: 1 },
            ]
        )
    }

    #[test]
    fn extend_match() {
        let mut a = Match::with_matched(16, 3, vec![1, 2, 3]);
        let b = Match::with_matched(8, 3, vec![5, 6, 7]);

        let s = Scoring::default();

        a.extend_with(&b, &s);

        assert_eq!(a.score(), 24 - s.penalty_distance);
        assert_eq!(a.consecutive, 0);
        assert_eq!(a.matched_indices().len(), 6);
    }

    #[test]
    fn extend_match_cont() {
        let mut a = Match::with_matched(16, 3, vec![1, 2, 3]);
        let b = Match::with_matched(8, 3, vec![4, 5, 6]);

        let s = Scoring::default();

        a.extend_with(&b, &s);

        assert_eq!(a.score(), 16 + 8 + (3 + 3 + 1) * s.bonus_consecutive);
        assert_eq!(a.consecutive, 3 + 3 + 1);
        assert_eq!(a.matched_indices().len(), 6);
    }
}
