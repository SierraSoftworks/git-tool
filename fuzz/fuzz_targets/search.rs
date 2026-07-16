#![no_main]

use git_tool::search;
use libfuzzer_sys::fuzz_target;

mod common;

fuzz_target!(|data: &[u8]| {
    let mut fields = common::fields(data).into_iter();
    let pattern = fields.next().unwrap_or_default();
    let mut values = fields.collect::<Vec<_>>();
    values.push(pattern.clone());

    let filtered = search::matches(&pattern, values.clone());
    let ranked = search::best_matches(&pattern, values.clone());
    let matched_by_any = search::matches_any(&[pattern.as_str()], values.clone());

    assert_eq!(filtered, matched_by_any);
    assert!(search::fuzzy_matches(&pattern, &pattern));
    assert!(filtered.contains(&pattern));

    let mut filtered_sorted = filtered;
    let mut ranked_sorted = ranked.clone();
    filtered_sorted.sort();
    ranked_sorted.sort();
    assert_eq!(filtered_sorted, ranked_sorted);
});
