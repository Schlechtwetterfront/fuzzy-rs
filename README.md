# sublime_fuzzy

Fuzzy matching algorithm based on Sublime Text's string search. Iterates through characters of a search string and calculates a score based on matching consecutive/close groups of characters. Tries to find the best match by scoring multiple match paths.

### Documentation

Check out the documentation at [docs.rs](https://docs.rs/sublime_fuzzy/0.2.0/sublime_fuzzy/).

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

Simple formatting:

```rust
use sublime_fuzzy::{best_match, format_simple};

let s = "some search thing";
let search = "something";
let result = best_match(search, s).unwrap();

// Output: <span>some</span> search <span>thing</span>
println!("formatted: {:?}", format_simple(&result, s, "<span>", "</span>"));
```

Adjust scoring:

```rust
use sublime_fuzzy::FuzzySearcher;

let mut search = FuzzySearcher::new();

search.set_search("something");
search.set_target("some search thing");

// Weight consecutive matching chars less.
search.set_score_consecutive(4);

println!("result: {:?}", search.best_match());
```
