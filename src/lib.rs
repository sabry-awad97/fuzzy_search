/// Generates a regex pattern for fuzzy searching a given search term.
///
/// This function creates a case-insensitive pattern that:
/// - Splits the search term into words
/// - Matches characters in sequence with flexible spacing
/// - Allows for typos and partial matches
/// - Handles short words (≤3 chars) and long words differently
///
/// # Examples
///
/// ```
/// use fuzzy_search::fuzzy_search_pattern;
/// let pattern = fuzzy_search_pattern("hello world");
/// let regex = regex::Regex::new(&pattern).unwrap();
/// assert!(regex.is_match("hello world"));
/// assert!(regex.is_match("HELLO WORLD"));
/// assert!(regex.is_match("hello there world"));
/// ```
pub fn fuzzy_search_pattern(search_term: &str) -> String {
    let words: Vec<String> = search_term
        .split_whitespace()
        .map(|word| {
            let chars: Vec<_> = word
                .chars()
                .map(|c| regex::escape(&c.to_string()))
                .collect();

            if chars.len() <= 3 {
                format!("(?:[^\\s]*{}[^\\s]*)", chars.join("[^\\s]*"))
            } else {
                let required_count = chars.len() / 2;
                format!(
                    "(?:[^\\s]*(?:{})[^\\s]*)",
                    chars[..required_count].join("[^\\s]*")
                )
            }
        })
        .collect();

    format!("(?i).*{}.*", words.join(".*"))
}

#[cfg(test)]
mod tests {
    use super::fuzzy_search_pattern;
    use regex::Regex;

    #[test]
    fn test_single_word_pattern() {
        let pattern = fuzzy_search_pattern("hello");
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello"));
        assert!(regex.is_match("HELLO"));
        assert!(regex.is_match("hello world"));
        assert!(regex.is_match("say hello there"));
        assert!(regex.is_match("heeello")); // with extra chars
    }

    #[test]
    fn test_multi_word_pattern() {
        let pattern = fuzzy_search_pattern("hello world");
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello world"));
        assert!(regex.is_match("HELLO WORLD"));
        assert!(regex.is_match("hello there world"));
        assert!(regex.is_match("My hello to the world"));
        assert!(!regex.is_match("hello")); // missing second word
        assert!(!regex.is_match("world")); // missing first word
    }

    #[test]
    fn test_short_word_pattern() {
        let pattern = fuzzy_search_pattern("hi");
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hi"));
        assert!(regex.is_match("HI"));
        assert!(regex.is_match("this"));
        assert!(regex.is_match("history"));
        assert!(regex.is_match("hi there"));
    }

    #[test]
    fn test_long_word_pattern() {
        let pattern = fuzzy_search_pattern("programming");
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("programming"));
        assert!(regex.is_match("PROGRAMMING"));
        assert!(regex.is_match("programmming")); // typo
        assert!(regex.is_match("program")); // partial match is ok
    }

    #[test]
    fn test_case_sensitivity() {
        let pattern = fuzzy_search_pattern("Test");
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("Test"));
        assert!(regex.is_match("test"));
        assert!(regex.is_match("TEST"));
        assert!(regex.is_match("testing"));
    }
}
