# sublime_fuzzy [![sublime_fuzzy on crates.io](https://img.shields.io/crates/v/sublime_fuzzy.svg)](https://crates.io/crates/sublime_fuzzy)

Fuzzy matching algorithm based on Sublime Text's string search. Iterates through
characters of a search string and calculates a score based on matching
consecutive/close groups of characters.

### Usage

Basic usage:

```rust
use sublime_fuzzy::best_match;

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

// Output: score: 368
println!("score: {:?}", result.score());
```

`Match.continuous_matches()` returns a list of consecutive matches
(`(start_index, length)`). Based on those the input string can be formatted.
`sublime_fuzzy` provides a simple formatting function that wraps matches in
tags.

```rust
use sublime_fuzzy::{best_match, format_simple};

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

// Output: <span>some</span> search <span>thing</span>
println!("formatted: {:?}", format_simple(&result, s, "<span>", "</span>"));
```

Matches are scored based on consecutively matching chars (bonus) and distance
between two chars (penalty). The actual values can be adjusted.

```rust
use sublime_fuzzy::{FuzzySearch, ScoreConfig};

let mut search = FuzzySearch::new("something", "some search thing");

let config = ScoreConfig {
    bonus_consecutive: 20,
    penalty_distance: 8
};
// Weight consecutive matching chars less.
search.set_score_config(config);

println!("result: {:?}", search.best_match());
```

**Note:** This module removes any whitespace in the pattern (`'something'`
in the examples above). It does not apply any other formatting. Lowercasing
the inputs for example has to be done manually.

### Documentation

Check out the documentation at [docs.rs](https://docs.rs/sublime_fuzzy/).
