//! Very basic binary for using/testing `sublime_fuzzy` from the command line.
//!
//! Pass the query as first and target string as second parameters to `sfz`.
use std::env;

use sublime_fuzzy::{best_match, format_simple};

extern crate sublime_fuzzy;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let q = args.get(1).expect("Missing query arg");
    let s = args.get(2).expect("Missing target arg");

    if let Some(m) = best_match(q, s) {
        println!("{}", format_simple(&m, s, "<", ">"));
    } else {
        println!("No match");
    }
}
