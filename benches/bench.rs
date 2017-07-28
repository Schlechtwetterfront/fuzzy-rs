#![feature(test)]

extern crate test;
extern crate sublime_fuzzy;

use test::Bencher;

use sublime_fuzzy::{best_match};

#[bench]
fn empty(b: &mut Bencher) {
    b.iter(|| 1);
}

#[bench]
fn short(b: &mut Bencher) {
    b.iter(|| best_match("jelly", "jellyfish"));
}

#[bench]
fn medium_start(b: &mut Bencher) {
    b.iter(|| best_match(
        "tracking",
        "This is a tracking issue for the #[bench] attribute and its stability in the compiler. Currently it is not possible to use this from stable Rust as it requires extern crate test which is itself not stable."
    ));
}

#[bench]
fn medium_middle(b: &mut Bencher) {
    b.iter(|| best_match(
        "requires",
        "This is a tracking issue for the #[bench] attribute and its stability in the compiler. Currently it is not possible to use this from stable Rust as it requires extern crate test which is itself not stable."
    ));
}

#[bench]
fn medium_end(b: &mut Bencher) {
    b.iter(|| best_match(
        "itself",
        "This is a tracking issue for the #[bench] attribute and its stability in the compiler. Currently it is not possible to use this from stable Rust as it requires extern crate test which is itself not stable."
    ));
}

#[bench]
fn long_start_close(b: &mut Bencher) {
    b.iter(|| best_match(
        "empty baseline",
        r"The empty benchmark is there as a baseline. An anecdote: In my first
          compilation of the benchmark, I forgot to add -O to the rustc command
          line, and wound up with a few ns/iter on an empty benchmark. Thus, I
          now always have an empty benchmark in my list, to make sure I benchmark
          an optimized version."
    ));
}

#[bench]
fn long_middle_close(b: &mut Bencher) {
    b.iter(|| best_match(
        "rustc wound",
        r"The empty benchmark is there as a baseline. An anecdote: In my first
          compilation of the benchmark, I forgot to add -O to the rustc command
          line, and wound up with a few ns/iter on an empty benchmark. Thus, I
          now always have an empty benchmark in my list, to make sure I benchmark
          an optimized version."
    ));
}
