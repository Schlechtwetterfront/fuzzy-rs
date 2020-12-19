use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub type CharSet = HashSet<char>;
pub type Occurrences = HashMap<char, Vec<Occurrence>>;

#[derive(Clone, Debug)]
pub struct Occurrence {
    pub target_idx: usize,
    pub is_start: bool,
    pub char: char,
}

impl Eq for Occurrence {}

impl PartialEq for Occurrence {
    fn eq(&self, other: &Occurrence) -> bool {
        self.target_idx == other.target_idx
            && self.char == other.char
            && self.is_start == other.is_start
    }
}

pub fn build_occurrences(query: &QueryChars, string: &str, case_insensitive: bool) -> Occurrences {
    let query_chars = condense(query, case_insensitive);

    let mut occurrences = HashMap::new();

    let lower = string.to_lowercase();

    let mut prev_is_upper = false;
    let mut prev_is_sep = true;
    let mut prev_is_start = false;

    for (i, (lower_c, original_c)) in lower.chars().zip(string.chars()).enumerate() {
        let mut is_start = false;
        let is_sep = is_word_sep(original_c);
        let is_upper = original_c.is_uppercase();

        let key_char = if case_insensitive {
            lower_c
        } else {
            original_c
        };

        if is_sep {
            prev_is_upper = false;
            prev_is_sep = true;
            prev_is_start = false;

            if query_chars.contains(&key_char) {
                occurrences
                    .entry(key_char)
                    .or_insert(Vec::new())
                    .push(Occurrence {
                        char: original_c,
                        target_idx: i,
                        is_start,
                    });
            }

            continue;
        }

        if prev_is_sep {
            is_start = true;
        } else {
            if !prev_is_start && (prev_is_upper != is_upper) {
                is_start = true;
            }
        }

        if query_chars.contains(&key_char) {
            occurrences
                .entry(key_char)
                .or_insert(Vec::new())
                .push(Occurrence {
                    char: original_c,
                    target_idx: i,
                    is_start,
                });
        }

        prev_is_start = is_start;
        prev_is_sep = is_sep;
        prev_is_upper = is_upper;
    }

    occurrences
}

fn is_word_sep(c: char) -> bool {
    !c.is_alphanumeric()
}

fn condense(s: &QueryChars, case_insensitive: bool) -> CharSet {
    HashSet::from_iter(s.iter().map(|qc| {
        if case_insensitive {
            qc.lower
        } else {
            qc.original
        }
    }))
}

pub type QueryChars = Vec<QueryChar>;

#[derive(Clone, Debug)]
pub struct QueryChar {
    pub original: char,
    pub lower: char,
}

impl Eq for QueryChar {}

impl PartialEq for QueryChar {
    fn eq(&self, other: &QueryChar) -> bool {
        self.original == other.original && self.lower == other.lower
    }
}

pub fn process_query(query: &str) -> QueryChars {
    let lower_query = query.to_lowercase();

    query
        .chars()
        .zip(lower_query.chars())
        .filter_map(|(original, lower)| {
            if original.is_whitespace() {
                return None;
            }

            Some(QueryChar { original, lower })
        })
        .collect::<Vec<QueryChar>>()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    use super::{build_occurrences, condense, is_word_sep, process_query, Occurrence, QueryChar};

    #[test]
    fn word_seps() {
        let seps: Vec<char> = vec![
            '/', '\\', '|', '_', '-', ' ', '\t', ':', '.', ',', '~', '>', '<',
        ];

        assert!(seps.into_iter().all(|s| is_word_sep(s)));
    }

    #[test]
    fn condense_casing() {
        assert_eq!(
            condense(&process_query("SCC"), true),
            HashSet::from_iter(vec!['s', 'c']),
            "Query chars not lowercased"
        );
        assert_eq!(
            condense(&process_query("SCC"), false),
            HashSet::from_iter(vec!['S', 'C']),
            "Query chars not matching original case"
        );
    }

    #[test]
    fn query_processing() {
        assert_eq!(
            vec![
                QueryChar {
                    lower: 'a',
                    original: 'a'
                },
                QueryChar {
                    lower: 'b',
                    original: 'b'
                },
                QueryChar {
                    lower: 'c',
                    original: 'c'
                }
            ],
            process_query("a b c"),
            "Whitespace not removed"
        );

        assert_eq!(
            vec![
                QueryChar {
                    lower: 'a',
                    original: 'A'
                },
                QueryChar {
                    lower: 'b',
                    original: 'B'
                },
                QueryChar {
                    lower: 'c',
                    original: 'C'
                }
            ],
            process_query("ABC")
        );
    }

    #[test]
    fn occurrence_eq() {
        let a = Occurrence {
            char: 'c',
            target_idx: 0,
            is_start: true,
        };

        assert_eq!(
            a,
            Occurrence {
                char: 'c',
                target_idx: 0,
                is_start: true
            }
        );
        assert_ne!(
            a,
            Occurrence {
                char: 'c',
                target_idx: 0,
                is_start: false
            },
            "is_start differs but eq"
        );
        assert_ne!(
            a,
            Occurrence {
                char: 'c',
                target_idx: 1,
                is_start: true
            },
            "target_idx differs but eq"
        );

        assert_ne!(
            a,
            Occurrence {
                char: 'b',
                target_idx: 0,
                is_start: true
            },
            "char differs but eq"
        );
    }

    #[test]
    fn occurrences() {
        let t = "SoccerCartoonController";

        let mut occs = build_occurrences(&process_query("scc"), t, true);

        assert_eq!(occs.len(), 2);

        let s = occs.remove(&'s').expect("Missing s occurrences");

        assert_eq!(
            s,
            vec![Occurrence {
                char: 'S',
                target_idx: 0,
                is_start: true,
            }]
        );

        let c = occs.remove(&'c').expect("Missing c occurrences");

        assert_eq!(
            c,
            vec![
                Occurrence {
                    char: 'c',
                    target_idx: 2,
                    is_start: false,
                },
                Occurrence {
                    char: 'c',
                    target_idx: 3,
                    is_start: false,
                },
                Occurrence {
                    char: 'C',
                    target_idx: 6,
                    is_start: true,
                },
                Occurrence {
                    char: 'C',
                    target_idx: 13,
                    is_start: true,
                },
            ]
        );
    }

    #[test]
    fn occurrences_2() {
        let t = "SccsCoolController";

        let mut occs = build_occurrences(&process_query("scc"), t, true);

        assert_eq!(occs.len(), 2);

        let s = occs.remove(&'s').expect("Missing s occurrences");

        assert_eq!(
            s,
            vec![
                Occurrence {
                    char: 'S',
                    target_idx: 0,
                    is_start: true,
                },
                Occurrence {
                    char: 's',
                    target_idx: 3,
                    is_start: false,
                }
            ]
        );

        let c = occs.remove(&'c').expect("Missing c occurrences");

        assert_eq!(
            c,
            vec![
                Occurrence {
                    char: 'c',
                    target_idx: 1,
                    is_start: false,
                },
                Occurrence {
                    char: 'c',
                    target_idx: 2,
                    is_start: false,
                },
                Occurrence {
                    char: 'C',
                    target_idx: 4,
                    is_start: true,
                },
                Occurrence {
                    char: 'C',
                    target_idx: 8,
                    is_start: true,
                },
            ]
        );
    }
}
