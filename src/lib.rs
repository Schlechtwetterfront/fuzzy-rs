#![allow(dead_code)]

use std::cmp::{max, Ordering};

#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub len: usize,
}

impl Match {
    pub fn new() -> Self {
        Match {
            start: 0,
            len: 0,
        }
    }

    pub fn with(start: usize, len: usize) -> Self {
        Match {
            start: start,
            len: len,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    score: isize,
    matches: Vec<Match>,
}

impl SearchResult {
    pub fn new() -> Self {
        SearchResult {
            score: 0,
            matches: Vec::new(),
        }
    }

    pub fn with(score: isize, matches: Vec<Match>) -> Self {
        SearchResult {
            score: score,
            matches: matches,
        }        
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &SearchResult) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &SearchResult) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchResult {}

impl PartialEq for SearchResult {
    fn eq(&self, other: &SearchResult) -> bool {
        self.score == other.score
    }
}

pub struct FuzzySearcher {
    score_distance: usize,
    score_found_char: usize,
    score_consecutive: usize,
    target: String,
    search: String,
}

impl FuzzySearcher {
    pub fn new() -> Self {
        FuzzySearcher {
            score_distance: 4,
            score_found_char: 16,
            score_consecutive: 16,
            target: String::new(),
            search: String::new(),
        }
    }

    pub fn set_score_distance(&mut self, score: usize) {
        self.score_distance = score;
    }

    pub fn set_score_found_char(&mut self, score: usize) {
        self.score_found_char = score;
    }

    pub fn set_score_consecutive(&mut self, score: usize) {
        self.score_consecutive = score;
    }

    pub fn set_target(&mut self, target: &str) {
        self.target = target.to_owned();
    }

    pub fn set_search(&mut self, search: &str) {
        self.search = search.to_owned();
    }

    pub fn chain_score(&mut self, match_chain: &Vec<(usize, char)>) -> SearchResult {
        let mut matches = Vec::new();
        let mut score: isize = 0;

        let mut consecutive_char_score = 0;

        let mut last_index = 0;

        let mut current_match = Match::new();

        let mut first_char = true;

        for &(i, _) in match_chain {
            if i == last_index && !first_char {
                consecutive_char_score += self.score_consecutive;
                current_match.len += 1;
            } else {
                if current_match.len > 0 {
                    matches.push(current_match);
                }
                consecutive_char_score = 0;
                current_match = Match::with(i, 1);
            }

            let mut dist: isize = 0;
            if !first_char {
                dist = max(i as isize - last_index as isize - 1, 0) as isize;
            }

            first_char = false;

            score -= dist * self.score_distance as isize;
            score += self.score_found_char as isize;
            score += consecutive_char_score as isize;

            last_index = i;
        }

        if current_match.len > 0 {
            matches.push(current_match);
        }

        SearchResult::with(score, matches)
    }

    pub fn best_match(&mut self) -> Option<SearchResult> {
        let chains = match_chains(&self.search, &self.target, 0, 0, &mut Vec::new());
        let mut results: Vec<SearchResult> = chains.iter().map(|x| self.chain_score(x)).collect();
        results.sort();

        if let Some(r) = results.first() {
            return Some(r.to_owned());
        }
        None
    }
}

fn occurences(what: char, target: &str, search_offset: usize) -> Option<Vec<usize>> {
    let mut occurences = Vec::new();
    let mut start_index = search_offset;
    loop {
        if let Some(next_start) = target.chars().skip(start_index).position(|x| x == what) {
            start_index = next_start + start_index + 1;
            occurences.push(next_start);
        } else {
            break;
        }
        println!("occurences");
    }
    if occurences.len() > 0 {
        Some(occurences)
    } else {
        None
    }
}

fn match_chains(search: &str, target: &str, search_offset: usize, mut index: usize, list: &Vec<(usize, char)>) -> Vec<Vec<(usize, char)>> {
    if index > target.len() - 1 {
        let mut container = Vec::new();
        container.push(list.clone());
        return container;
    }
    let mut search_char = search.chars().nth(index).unwrap();

    let occurences = loop {
        if let Some(result) = occurences(search_char, target, search_offset) {
            break result;
        } else {
            index += 1;
            if let Some(new_c) = search.chars().nth(index) {
                search_char = new_c;
            } else {
                let mut container = Vec::new();
                container.push(list.clone());
                return container;
            }
        }
    };

    let mut results: Vec<Vec<(usize, char)>> = Vec::new();
    for o in occurences {
        let mut list_cpy = list.clone();
        list_cpy.push((o, search_char));

        results.append(&mut match_chains(search, target, o + 1, index + 1, &mut list_cpy));
    }

    results
}

pub fn best_match(search: &str, target: &str) -> Option<SearchResult> {
    let mut searcher = FuzzySearcher::new();
    searcher.set_search(search);
    searcher.set_target(target);

    searcher.best_match()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use best_match;
        println!("result: {:?}", best_match("observablecollection", "obsinline"));
    }
}
