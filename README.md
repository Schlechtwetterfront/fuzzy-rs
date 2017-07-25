# sublime_fuzzy

Fuzzy matching algorithm based on Sublime Text's string search.

## Usage

Basic usage:

    let s = "some search thing";
    let search = "something";
    let result = best_match(search, s).unwrap();

Simple formatting:

    let s = "some search thing";
    let search = "something";
    let result = best_match(search, s).unwrap();
    
    // Will output '<span>some</span> search <span>thing</span>'.
    println!("formatted: {:?}", format_simple(&result, s, "<span>", "</span>"));

Adjust scoring:

    use sublime_fuzzy::FuzzySearcher;
    
    let mut search = FuzzySearcher::new();
    
    search.set_search("something");
    search.set_target("some search thing");
    
    // Weight consecutive matching chars less.
    search.set_score_consecutive(4);
    
    println!("result: {:?}", search.best_match());
