pub fn matches(value: &str, sequence: &str) -> bool {
    if sequence.len() > value.len() {
        return false;
    }

    let mut seq_iter = sequence.chars().peekable();
    for c in value.chars() {
        match seq_iter.peek() {
            Some(sc) => {
                if c.to_lowercase().eq(sc.to_lowercase()) {
                    seq_iter.next();
                }
            }
            None => break,
        }
    }

    match seq_iter.peek() {
        Some(_) => false,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_empty_sequence() {
        assert!(matches("", ""));
        assert!(matches("test", ""));
    }

    #[test]
    fn match_exact() {
        assert!(matches("test", "test"));
        assert!(!matches("test", "bucket"));
    }

    #[test]
    fn match_substring() {
        assert!(matches("test", "tes"));
        assert!(matches("test", "est"));
        assert!(!matches("test", "set"));
    }

    #[test]
    fn match_ordering() {
        assert!(matches("test", "tst"));
        assert!(matches("test", "st"));
        assert!(!matches("test", "se"));
    }

    #[test]
    fn match_too_long() {
        assert!(!matches("test", "testing"));
    }
}
