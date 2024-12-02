use std::error::Error;
use std::fmt;

/// Custom error types for fuzzy search operations
#[derive(Debug)]
pub enum FuzzyError {
    /// Invalid search pattern
    InvalidPattern(String),
    /// Regex compilation error
    RegexError(regex::Error),
}

impl fmt::Display for FuzzyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FuzzyError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            FuzzyError::RegexError(err) => write!(f, "Regex error: {}", err),
        }
    }
}

impl Error for FuzzyError {}

impl From<regex::Error> for FuzzyError {
    fn from(err: regex::Error) -> Self {
        FuzzyError::RegexError(err)
    }
}

/// Configuration options for fuzzy search
#[derive(Debug, Clone)]
pub struct FuzzyConfig {
    /// Minimum word length for applying typo tolerance
    pub min_word_length: usize,
    /// Required character ratio for longer words (0.0 to 1.0)
    pub required_char_ratio: f32,
    /// Whether to enable case-sensitive matching
    pub case_sensitive: bool,
    /// Maximum allowed character gap
    pub max_char_gap: usize,
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            min_word_length: 3,
            required_char_ratio: 0.5,
            case_sensitive: false,
            max_char_gap: 10,
        }
    }
}

/// A builder for creating customized fuzzy search patterns
#[derive(Debug)]
pub struct FuzzySearchBuilder {
    config: FuzzyConfig,
    search_term: String,
}

impl FuzzySearchBuilder {
    /// Create a new builder instance
    pub fn new(search_term: impl Into<String>) -> Self {
        Self {
            config: FuzzyConfig::default(),
            search_term: search_term.into(),
        }
    }

    /// Set minimum word length for typo tolerance
    pub fn min_word_length(mut self, length: usize) -> Self {
        self.config.min_word_length = length;
        self
    }

    /// Set required character ratio for longer words
    pub fn required_char_ratio(mut self, ratio: f32) -> Self {
        self.config.required_char_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Enable or disable case sensitivity
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.config.case_sensitive = sensitive;
        self
    }

    /// Set maximum character gap
    pub fn max_char_gap(mut self, gap: usize) -> Self {
        self.config.max_char_gap = gap;
        self
    }

    /// Build the regex pattern
    pub fn build(&self) -> Result<String, FuzzyError> {
        create_fuzzy_pattern(&self.search_term, &self.config)
    }

    /// Build and compile the regex
    pub fn compile(&self) -> Result<regex::Regex, FuzzyError> {
        let pattern = self.build()?;
        Ok(regex::Regex::new(&pattern)?)
    }
}

/// Creates a fuzzy search pattern with custom configuration
fn create_fuzzy_pattern(search_term: &str, config: &FuzzyConfig) -> Result<String, FuzzyError> {
    if search_term.trim().is_empty() {
        return Err(FuzzyError::InvalidPattern(
            "Search term cannot be empty".into(),
        ));
    }

    let words: Vec<String> = search_term
        .split_whitespace()
        .map(|word| create_word_pattern(word, config))
        .collect();

    let case_flag = if !config.case_sensitive { "(?i)" } else { "" };
    Ok(format!("{}.*{}.*", case_flag, words.join(".*")))
}

/// Creates a pattern for a single word
fn create_word_pattern(word: &str, config: &FuzzyConfig) -> String {
    let chars: Vec<_> = word
        .chars()
        .map(|c| regex::escape(&c.to_string()))
        .collect();

    if chars.len() <= config.min_word_length {
        format!(
            "(?:[^\\s]*{}[^\\s]*)",
            chars.join(&format!("[^\\s]{{0,{}}}", config.max_char_gap))
        )
    } else {
        let required_count = (chars.len() as f32 * config.required_char_ratio) as usize;
        format!(
            "(?:[^\\s]*(?:{})[^\\s]*)",
            chars[..required_count].join(&format!("[^\\s]{{0,{}}}", config.max_char_gap))
        )
    }
}

/// Simplified function for quick fuzzy pattern generation with default settings
pub fn fuzzy_search_pattern(search_term: &str) -> String {
    FuzzySearchBuilder::new(search_term)
        .build()
        .unwrap_or_else(|_| "".to_string())
}

#[cfg(test)]
mod tests {
    use super::FuzzySearchBuilder;
    use regex::Regex;

    #[test]
    fn test_single_word_pattern() {
        let pattern = FuzzySearchBuilder::new("hello").build().unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello"));
        assert!(regex.is_match("HELLO"));
        assert!(regex.is_match("hello world"));
        assert!(regex.is_match("say hello there"));
        assert!(regex.is_match("heeello")); // with extra chars
    }

    #[test]
    fn test_multi_word_pattern() {
        let pattern = FuzzySearchBuilder::new("hello world").build().unwrap();
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
        let pattern = FuzzySearchBuilder::new("hi").build().unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hi"));
        assert!(regex.is_match("HI"));
        assert!(regex.is_match("this"));
        assert!(regex.is_match("history"));
        assert!(regex.is_match("hi there"));
    }

    #[test]
    fn test_long_word_pattern() {
        let pattern = FuzzySearchBuilder::new("programming").build().unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("programming"));
        assert!(regex.is_match("PROGRAMMING"));
        assert!(regex.is_match("programmming")); // typo
        assert!(regex.is_match("program")); // partial match is ok
    }

    #[test]
    fn test_case_sensitivity() {
        let pattern = FuzzySearchBuilder::new("Test")
            .case_sensitive(true)
            .build()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("Test"));
        assert!(!regex.is_match("test"));
        assert!(!regex.is_match("TEST"));
        assert!(!regex.is_match("testing"));
    }

    #[test]
    fn test_custom_config() {
        let pattern = FuzzySearchBuilder::new("hello")
            .min_word_length(5)
            .required_char_ratio(0.8)
            .max_char_gap(2)
            .build()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello"));
        assert!(regex.is_match("heello")); // small gap
        assert!(!regex.is_match("h e l l o")); // too big gaps
    }

    #[test]
    fn test_empty_pattern() {
        let result = FuzzySearchBuilder::new("").build();
        assert!(matches!(result, Err(super::FuzzyError::InvalidPattern(_))));
    }

    #[test]
    fn test_whitespace_only_pattern() {
        let result = FuzzySearchBuilder::new("   ").build();
        assert!(matches!(result, Err(super::FuzzyError::InvalidPattern(_))));
    }

    #[test]
    fn test_builder_methods() {
        let builder = FuzzySearchBuilder::new("test")
            .min_word_length(4)
            .required_char_ratio(0.75)
            .case_sensitive(true)
            .max_char_gap(3);

        let pattern = builder.build().unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("test"));
        assert!(!regex.is_match("TEST")); // case sensitive
    }
}
