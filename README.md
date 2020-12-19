# sublime_fuzzy [![sublime_fuzzy on crates.io](https://img.shields.io/crates/v/sublime_fuzzy.svg)](https://crates.io/crates/sublime_fuzzy)

Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
characters of a search string and calculates a score.

The score is based on several factors:

- **Word starts** like the `t` in `some_thing` get a bonus (`bonus_word_start`)
- **Consecutive matches** get an accumulative bonus for every consecutive match (`bonus_consecutive`)
- Matches that also match **case** (`T` -> `T` instead of `t` -> `T`) in case of a case-insensitive search get a bonus (`bonus_match_case`)
- The **distance** between two matches will be multiplied with the `penalty_distance` penalty and subtracted from the score

The default scoring is configured to give a lot of weight to word starts. So a pattern `scc` will match
**S**occer**C**artoon**C**ontroller, not **S**o**cc**erCartoonController.

# Match Examples

With default weighting.

| Pattern     | Target string             | Result                              |
| ----------- | ------------------------- | ----------------------------------- |
| `scc`       | `SoccerCartoonController` | **S**occer**C**artoon**C**ontroller |
| `something` | `some search thing`       | **some** search **thing**           |

# Usage

Basic usage:

```rust
use sublime_fuzzy::best_match;

let result = best_match("something", "some search thing");

assert!(result.is_some());
```

`Match::continuous_matches` returns an iter of consecutive matches. Based on those the input
string can be formatted.

`format_simple` provides a simple formatting that wraps matches in tags:

```rust
use sublime_fuzzy::{best_match, format_simple};

let target = "some search thing";

let result = best_match("something", target).unwrap();

assert_eq!(
    format_simple(&result, target, "<span>", "</span>"),
    "<span>some</span> search <span>thing</span>"
);
```

The weighting of the different factors can be adjusted:

```rust
use sublime_fuzzy::{FuzzySearch, Scoring};

// Or pick from one of the provided `Scoring::...` methods like `emphasize_word_starts`
let scoring = Scoring {
    bonus_consecutive: 128,
    bonus_word_start: 0,
    ..Scoring::default()
};

let result = FuzzySearch::new("something", "some search thing")
    .case_sensitive()
    .score_with(&scoring)
    .best_match();

assert!(result.is_some())
```

**Note:** Any whitespace in the pattern (`'something'`
in the examples above) will be removed.

### Documentation

Check out the documentation at [docs.rs](https://docs.rs/sublime_fuzzy/).
