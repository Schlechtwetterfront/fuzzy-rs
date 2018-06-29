use std::cmp::Ordering;

/// A container holding the boni and penalties used to score a match.
///
/// # Examples
///
/// Don't give a bonus for matching word starts (like `T` in `SomeThing`).
///
///     use sublime_fuzzy::ScoreConfig;
///     
///     let score_cfg = ScoreConfig {
///         bonus_word_start: 0,
///         ..Default::default()
///     };
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ScoreConfig {
    pub bonus_consecutive: isize,
    pub bonus_word_start: isize,
    pub bonus_coverage: isize,
    pub penalty_distance: isize,
}

impl ScoreConfig {
    /// Creates a new `ScoreConfig` by specifying all boni/penalties.
    pub fn new(
        bonus_consecutive: isize,
        bonus_word_start: isize,
        bonus_coverage: isize,
        penalty_distance: isize,
    ) -> Self {
        ScoreConfig {
            bonus_consecutive: bonus_consecutive,
            bonus_word_start: bonus_word_start,
            bonus_coverage: bonus_coverage,
            penalty_distance: penalty_distance,
        }
    }
}

impl Default for ScoreConfig {
    fn default() -> Self {
        ScoreConfig::new(8, 72, 64, 4)
    }
}

/// Intermediate score used for scoring parts of the patterns.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Score {
    pub score: isize,
    pub consecutive_matches: usize,
    pub matches: Vec<usize>,
}

impl Score {
    pub fn new(score: isize, consec: usize, matches: Vec<usize>) -> Self {
        Score {
            score: score,
            consecutive_matches: consec,
            matches: matches,
        }
    }

    /// Assimilates another `Score` into this one.
    pub fn extend(&mut self, mut other: Score, cfg: &ScoreConfig) {
        // println!("Extending {:?} with {:?}", self, other);
        self.score += other.score;
        self.consecutive_matches += other.consecutive_matches;

        if let (Some(last), Some(first)) = (self.matches.last(), other.matches.first()) {
            let distance = first - last;

            match distance {
                0 => {}
                1 => {
                    self.consecutive_matches += 1;
                    self.score += self.consecutive_matches as isize * cfg.bonus_consecutive;
                }
                _ => {
                    self.consecutive_matches = 0;
                    let penalty = (distance as isize - 1) * cfg.penalty_distance;
                    // println!("Subtracting {}", penalty);
                    self.score -= penalty;
                }
            }
        }

        self.matches.append(&mut other.matches);
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Score {}

impl PartialEq for Score {
    fn eq(&self, other: &Score) -> bool {
        self.score == other.score
    }
}

pub fn consecutive_score(count: usize, score_config: &ScoreConfig) -> isize {
    count as isize * score_config.bonus_consecutive
}
