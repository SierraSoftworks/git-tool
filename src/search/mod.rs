pub fn matches(value: &str, sequence: &str) -> bool {
    if sequence.len() > value.len() {
        return false
    }

    let mut seq_iter = sequence.chars().peekable();
    for c in value.chars() {
        match seq_iter.peek() {
            Some(sc) => {
                if c.to_lowercase().eq(sc.to_lowercase()) {
                    seq_iter.next();
                }
            },
            None => break
        }
    }

    match seq_iter.peek() {
        Some(_) => false,
        None => true
    }
}