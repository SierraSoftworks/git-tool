use itertools::Itertools;

pub fn best_matches<T>(sequence: &str, values: T) -> Vec<T::Item>
where
    T: IntoIterator,
    T::Item: AsRef<str> + Clone,
{
    let matcher = SequenceMatcher::new(sequence);
    matcher.order_by(values, |v| v.to_owned())
}

pub fn best_matches_by<'a, T, F, K>(sequence: &str, values: T, to_key: F) -> Vec<T::Item>
where
    T: IntoIterator,
    T::Item: Clone + 'a,
    F: Fn(&T::Item) -> K,
    K: AsRef<str>,
{
    let matcher = SequenceMatcher::new(sequence);
    matcher.order_by(values, to_key)
}

#[cfg(test)]
fn score<T: AsRef<str>>(value: T, sequence: &str) -> Option<f32> {
    let matcher = SequenceMatcher::new(sequence);
    matcher.score(value)
}

struct SequenceMatcher<'a> {
    pattern: &'a str,
}

impl<'a> SequenceMatcher<'a> {
    pub fn new(pattern: &'a str) -> Self {
        Self { pattern }
    }

    pub fn order_by<'b, T, F, K>(&self, values: T, to_key: F) -> Vec<T::Item>
    where
        T: IntoIterator,
        T::Item: Clone + 'b,
        F: Fn(&T::Item) -> K,
        K: AsRef<str>,
    {
        if self.pattern.is_empty() {
            return values.into_iter().collect();
        }

        values
            .into_iter()
            .map(|v| (v.clone(), self.score(to_key(&v))))
            .filter(|(_, score)| score.is_some())
            .map(|(item, score)| (item, score.unwrap()))
            .sorted_unstable_by_key(|(v, _score)| to_key(v).as_ref().len())
            .sorted_by(|(_, score1), (_, score2)| {
                score2
                    .partial_cmp(score1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(item, _)| item)
            .collect()
    }

    pub fn score<T: AsRef<str>>(&self, value: T) -> Option<f32> {
        let value = value.as_ref();

        if value.is_empty() {
            return None;
        }

        if self.pattern.len() > value.len() {
            return None;
        }

        let mut shortest_sequence: Option<usize> = None;

        // Outer loop evaluates an offset from the end of the string to start searching
        for offset in 0..value.len() - 1 {
            let pattern = self.pattern.chars();
            let sequence = value.chars().skip(offset);

            shortest_sequence = match (shortest_sequence, self.score_sequence(sequence, pattern)) {
                (Some(best), Some(current)) => Some(if best < current { best } else { current }),
                (None, Some(current)) => Some(current),
                (_, None) => {
                    break;
                }
            }
        }

        shortest_sequence.map(|v| self.pattern.len() as f32 / v as f32)
    }

    fn score_sequence<S, P>(&self, sequence: S, pattern: P) -> Option<usize>
    where
        S: IntoIterator<Item = char>,
        P: IntoIterator<Item = char>,
    {
        // We work backwards because we know that the end of the pattern is more important than the start
        let mut pattern_iter = pattern.into_iter().peekable();

        let mut index = 0;

        let mut match_min_index = None;
        let mut match_max_index = None;

        for c in sequence.into_iter() {
            match pattern_iter.peek().map(|v| v.to_ascii_lowercase()) {
                Some(pattern_char) if pattern_char == c.to_ascii_lowercase() => {
                    // Mark this as the end of the match
                    match_min_index = match_min_index.or(Some(index));

                    // Move onto matching the next item
                    pattern_iter.next();
                }
                None => {
                    // If we reach the end of the pattern, then stop here
                    match_max_index = Some(index);
                    break;
                }
                _ => {}
            };
            index += 1;
        }

        // If we've reached the end of our sequence and the end of the pattern simultaneously, then mark the end index
        if let (None, None) = (match_max_index, pattern_iter.peek()) {
            match_max_index = Some(index)
        }

        match (match_min_index, match_max_index) {
            (Some(min_index), Some(max_index)) => Some(max_index - min_index),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_empty_sequence() {
        assert_eq!(score("", ""), None);
        assert_eq!(score("test", ""), None);
    }

    fn test_relative_scoring(pattern: &str, better: &str, worse: &str) {
        let worse_score = score(worse, pattern);
        let better_score = score(better, pattern);

        if let Some(better_score) = better_score {
            if let Some(worse_score) = worse_score {
                assert!(
                    better_score > worse_score,
                    "the pattern '{pattern}' should score better against '{better}' than '{worse}'"
                );
            }
        }
    }

    #[test]
    fn score_comparisons() {
        test_relative_scoring("test", "testing", "tea string");
        test_relative_scoring("tst", "test", "transit");
        test_relative_scoring(
            "sierralib",
            "SierraSoftworks/SierraLib.Analytics",
            "SierraSoftworks/libUpdate",
        );
    }

    #[test]
    fn score_ordering() {
        assert_eq!(
            best_matches("tst", vec!["tea string", "testing", "blob"]),
            vec!["testing", "tea string"],
            "the best matches method should order the list of items by the best match first"
        );

        assert_eq!(
            best_matches(
                "ghspt1",
                vec![
                    "gh:sierrasoftworks/test1",
                    "gh:sierrasoftworks/test2",
                    "gh:spartan563/test1",
                    "gh:spartan563/test2"
                ]
            ),
            vec!["gh:spartan563/test1"]
        );

        assert_eq!(
            best_matches("main", vec!["main123", "main", "main456",]),
            vec!["main", "main123", "main456"]
        );
    }
}
