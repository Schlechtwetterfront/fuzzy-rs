pub static DEFAULT_SCORING: Scoring = Scoring {
    bonus_consecutive: 8,
    bonus_word_start: 72,
    bonus_match_case: 8,
    penalty_distance: 4,
};

/// Bonuses/penalties used for scoring a [`Match`](crate::matching::Match).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Scoring {
    /// `current_consecutive_count * bonus_consecutive` will be added for every
    /// consecutive char match.
    ///
    /// `1 * bonus` for the first consecutive match, `2 * bonus` for
    /// the second, etc.
    pub bonus_consecutive: isize,
    /// Added when a query char matches a word start.
    pub bonus_word_start: isize,
    /// Added when the matched query char also matches the case of the target char.
    ///
    /// Only applied if the search is case insensitive.
    pub bonus_match_case: isize,
    /// Subtracted from the score for every char between two matches.
    pub penalty_distance: isize,
}

impl Scoring {
    /// Creates a new configuration with the given bonuses/penalties.
    pub fn new(
        bonus_consecutive: isize,
        bonus_word_start: isize,
        bonus_match_case: isize,
        penalty_distance: isize,
    ) -> Self {
        Scoring {
            bonus_consecutive,
            bonus_word_start,
            bonus_match_case,
            penalty_distance,
        }
    }

    /// Creates a configuration that emphasizes matching word starts (this is also the default).
    pub fn emphasize_word_starts() -> Self {
        Self::default()
    }

    /// Creates a configuration that emphasizes short distances between matched chars.
    pub fn emphasize_distance() -> Self {
        Scoring::new(12, 24, 8, 8)
    }
}

impl Default for Scoring {
    /// Creates a default configuration, see [`Scoring::emphasize_word_starts`].
    fn default() -> Self {
        DEFAULT_SCORING.clone()
    }
}
