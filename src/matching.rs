use std::cmp::Ordering;

/// Contains the calculated match score and all matches for a single result.
///
/// The score is not clamped to any range and in some cases will be negative.
/// Matches can be directly compared to find the target string that matches the
/// pattern the best.
///
/// The actual matched characters are stored as indices into the target string.
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
