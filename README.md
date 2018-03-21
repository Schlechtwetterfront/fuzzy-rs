# sublime_fuzzy [![sublime_fuzzy on crates.io](https://img.shields.io/crates/v/sublime_fuzzy.svg)](https://crates.io/crates/sublime_fuzzy)

Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
characters of a search string and calculates a score.

The score is based on several factors:
* **Word starts** like the `t` in `some_thing` get a bonus (`bonus_word_start`)
* **Consecutive matches** get an accumulative bonus for every consecutive match (`bonus_consecutive`)
* Matches with higher **coverage** (targets `some_release` (lower) versus `a_release` (higher) with pattern
`release`) will get a bonus multiplied by the coverage percentage (`bonus_coverage`)
* The **distance** between two matches will be multiplied with the `penalty_distance` penalty and subtracted from
the score

The default bonus/penalty values are set to give a lot of weight to word starts. So a pattern `scc` will match
**S**occer**C**artoon**C**ontroller, not **S**o**cc**erCartoonController.

# Usage

Basic usage:

```rust
use sublime_fuzzy::best_match;

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

println!("score: {:?}", result.score());
```

`Match.continuous_matches()` returns a list of consecutive matches
(`(start_index, length)`). Based on those the input string can be formatted.

`sublime_fuzzy` provides a simple formatting function that wraps matches in
tags:

```rust
use sublime_fuzzy::{best_match, format_simple};

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

assert_eq!(
    format_simple(&result, s, "<span>", "</span>"),
    "<span>some</span> search <span>thing</span>"
);
```

The weighting of the different factors can be adjusted:

```rust
use sublime_fuzzy::{FuzzySearch, ScoreConfig};

let case_insensitive = true;

let mut search = FuzzySearch::new("something", "some search thing", case_insensitive);

let config = ScoreConfig {
    bonus_consecutive: 12,
    bonus_word_start: 64,
    bonus_coverage: 64,
    penalty_distance: 4,
};

search.set_score_config(config);

println!("result: {:?}", search.best_match());
```

**Note:** Any whitespace in the pattern (`'something'`
in the examples above) will be removed.


### Documentation

Check out the documentation at [docs.rs](https://docs.rs/sublime_fuzzy/).
