use log::{debug, error, warn};
/// A Rust library for fuzzy text searching with regex pattern generation.
///
/// This library provides flexible pattern matching that's tolerant to typos
/// and variations in text. It uses a builder pattern with compile-time validation
/// for configuration.
///
/// # Examples
///
/// ```
/// use fuzzy_search::FuzzyConfig;
///
/// let config = FuzzyConfig::builder()
///     .search_term("hello")
///     .build();
///
/// let pattern = config.build_pattern().unwrap();
/// let regex = regex::Regex::new(&pattern).unwrap();
/// assert!(regex.is_match("hello"));
/// assert!(regex.is_match("heello")); // small gap
/// ```
///
/// Advanced usage with configuration:
/// ```
/// use fuzzy_search::FuzzyConfig;
///
/// let config = FuzzyConfig::builder()
///     .search_term("hello")
///     .case_sensitive(true)
///     .max_char_gap(1)
///     .min_word_length(3)
///     .required_char_ratio(0.8)
///     .build();
///
/// let pattern = config.build_pattern().unwrap();
/// let regex = regex::Regex::new(&pattern).unwrap();
/// assert!(regex.is_match("hello"));
/// assert!(regex.is_match("heello")); // small gap
/// ```
use std::error::Error;
use std::fmt;
use typed_builder::TypedBuilder;

/// Custom error types for fuzzy search operations
#[derive(Debug)]
pub enum FuzzyError {
    /// Invalid search pattern
    InvalidPattern(String),
    /// Regex compilation error
    RegexError(regex::Error),
    /// Empty pattern
    EmptyPattern,
}

impl fmt::Display for FuzzyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FuzzyError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            FuzzyError::RegexError(err) => write!(f, "Regex error: {}", err),
            FuzzyError::EmptyPattern => write!(f, "Empty pattern"),
        }
    }
}

impl Error for FuzzyError {}

impl From<regex::Error> for FuzzyError {
    fn from(err: regex::Error) -> Self {
        error!("Regex error: {}", err);
        FuzzyError::InvalidPattern(err.to_string())
    }
}

/// Configuration options for fuzzy search pattern generation
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc)]
pub struct FuzzyConfig {
    /// Search term to create pattern for
    #[builder(setter(into))]
    search_term: String,

    /// Minimum word length for applying typo tolerance
    #[builder(default = 3)]
    min_word_length: usize,

    /// Required character ratio for longer words (0.0 to 1.0)
    #[builder(default = 0.5, setter(transform = |v: f32| v.clamp(0.0, 1.0)))]
    required_char_ratio: f32,

    /// Whether to enable case-sensitive matching
    #[builder(default = false)]
    case_sensitive: bool,

    /// Maximum allowed character gap
    #[builder(default = 10)]
    max_char_gap: usize,
}

impl FuzzyConfig {
    /// Creates a pattern based on the configuration
    pub fn build_pattern(&self) -> Result<String, FuzzyError> {
        create_fuzzy_pattern(&self.search_term, self)
    }

    /// Creates and compiles a regex based on the configuration
    pub fn compile(&self) -> Result<regex::Regex, FuzzyError> {
        let pattern = self.build_pattern()?;
        Ok(regex::Regex::new(&pattern)?)
    }
}

/// Creates a fuzzy search pattern with custom configuration
fn create_fuzzy_pattern(search_term: &str, config: &FuzzyConfig) -> Result<String, FuzzyError> {
    // Validate search term
    if search_term.trim().is_empty() {
        error!("Empty search term provided");
        return Err(FuzzyError::EmptyPattern);
    }

    // Split search term into words
    let words: Vec<_> = search_term
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        error!("No valid words found in search term");
        return Err(FuzzyError::EmptyPattern);
    }

    // Check minimum word length requirement
    if words
        .iter()
        .any(|w| w.chars().count() < config.min_word_length)
    {
        warn!(
            "Words shorter than minimum length {}: {:?}",
            config.min_word_length, words
        );
    }

    // Split on whitespace but preserve punctuation
    let words: Vec<String> = words
        .into_iter()
        .map(|word| {
            if word.chars().any(|c| c.is_ascii_punctuation()) {
                // For words with punctuation, create a pattern that allows matching with or without the punctuation
                let parts: Vec<String> = word
                    .split(|c: char| c.is_ascii_punctuation())
                    .filter(|s| !s.is_empty())
                    .map(|part| create_word_pattern(part, config))
                    .collect();
                parts.join("[\\s\\p{Z}\\p{C}]*")
            } else {
                create_word_pattern(word, config)
            }
        })
        .collect();

    let case_flag = if !config.case_sensitive { "(?i)" } else { "" };
    // For multiple words, require all words to be present with flexible whitespace
    if words.len() > 1 {
        Ok(format!(
            "{}(?s).*?{}.*?",
            case_flag,
            words.join("[\\s\\p{Z}\\p{C}]+.*?")
        ))
    } else {
        Ok(format!("{}(?s).*?{}.*?", case_flag, words[0]))
    }
}

/// Creates a pattern for a single word
fn create_word_pattern(word: &str, config: &FuzzyConfig) -> String {
    debug!("Creating pattern for word: {}", word);
    debug!(
        "Config: max_char_gap={}, min_word_length={}, required_char_ratio={}",
        config.max_char_gap, config.min_word_length, config.required_char_ratio
    );

    // Special handling for single character inputs
    if word.chars().count() == 1 {
        let char_pattern = regex::escape(word);
        debug!("Single character pattern: {}", char_pattern);
        return format!("(?:[^\\s]*?{}[^\\s]*?)", char_pattern);
    }

    let chars: Vec<_> = word
        .chars()
        .map(|c| {
            let escaped = regex::escape(&c.to_string());
            if c.is_ascii_punctuation() || c.is_ascii_digit() || !c.is_ascii() {
                debug!("Special character '{}' escaped as: {}", c, escaped);
                format!("(?:{})?", escaped)
            } else if config.case_sensitive {
                debug!("Case-sensitive character '{}' escaped as: {}", c, escaped);
                escaped
            } else {
                debug!(
                    "Case-insensitive character '{}' pattern: [{}{}]",
                    c,
                    c.to_lowercase(),
                    c.to_uppercase()
                );
                format!("[{}{}]", c.to_lowercase(), c.to_uppercase())
            }
        })
        .collect();
    debug!("Processed chars: {:?}", chars);

    // Create gap patterns based on configuration
    let between_pattern = if config.max_char_gap > 0 {
        // When max_char_gap is set, allow any characters within the limit
        if config.max_char_gap > 10 {
            // For large gaps, allow any characters including spaces
            debug!(
                "Using large gap pattern with max_char_gap={}",
                config.max_char_gap
            );
            format!(".{{0,{}}}", config.max_char_gap)
        } else {
            // For small gaps, only allow non-space characters
            debug!(
                "Using small gap pattern with max_char_gap={}",
                config.max_char_gap
            );
            format!("[^\\s]{{0,{}}}", config.max_char_gap)
        }
    } else {
        // When max_char_gap is 0, don't allow any characters between
        debug!("Using zero gap pattern");
        "".to_string()
    };

    debug!("Between pattern: {}", between_pattern);

    // For high required_char_ratio, enforce stricter matching but still allow some flexibility
    let char_pattern = if config.required_char_ratio > 0.9 {
        debug!(
            "Using strict pattern with required_char_ratio={}",
            config.required_char_ratio
        );
        // Require all characters with optional gaps
        let mut pattern = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 {
                pattern.push_str(&between_pattern);
            }
            pattern.push_str(c);
        }
        pattern
    } else {
        // Allow flexible matching based on word length and required ratio
        let required_chars = (chars.len() as f32 * config.required_char_ratio).ceil() as usize;
        debug!(
            "Using flexible pattern with required_char_ratio={}, required_chars={}",
            config.required_char_ratio, required_chars
        );
        let (required, optional) = chars.split_at(required_chars);

        let mut pattern = String::new();
        // Add required characters with flexible gaps
        for (i, c) in required.iter().enumerate() {
            if i > 0 {
                pattern.push_str(&between_pattern);
            }
            pattern.push_str(c);
        }

        // Add optional characters
        if !optional.is_empty() {
            debug!("Adding {} optional characters", optional.len());
            pattern.push_str("(?:");
            for (i, c) in optional.iter().enumerate() {
                if i > 0 {
                    pattern.push_str(&between_pattern);
                }
                pattern.push_str(&format!("{}?", c));
            }
            pattern.push_str(")?");
        }
        pattern
    };

    // Create the final pattern with appropriate word boundaries
    let final_pattern = format!("(?:{})", char_pattern);
    debug!("Final word pattern: {}", final_pattern);
    final_pattern
}

/// Simplified function for quick fuzzy pattern generation with default settings
pub fn fuzzy_search_pattern(search_term: &str) -> String {
    FuzzyConfig::builder()
        .search_term(search_term)
        .build()
        .build_pattern()
        .unwrap_or_else(|_| "".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_single_word_pattern() {
        let pattern = FuzzyConfig::builder()
            .search_term("hello")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello"));
        assert!(regex.is_match("HELLO"));
        assert!(regex.is_match("hello world"));
        assert!(regex.is_match("say hello there"));
        assert!(regex.is_match("heeello")); // with extra chars
    }

    #[test]
    fn test_multi_word_pattern() {
        let pattern = FuzzyConfig::builder()
            .search_term("hello world")
            .build()
            .build_pattern()
            .unwrap();
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
        let pattern = FuzzyConfig::builder()
            .search_term("hi")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hi"));
        assert!(regex.is_match("HI"));
        assert!(regex.is_match("this"));
        assert!(regex.is_match("history"));
        assert!(regex.is_match("hi there"));
    }

    #[test]
    fn test_long_word_pattern() {
        let pattern = FuzzyConfig::builder()
            .search_term("programming")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("programming"));
        assert!(regex.is_match("PROGRAMMING"));
        assert!(regex.is_match("programmming")); // typo
        assert!(regex.is_match("program")); // partial match is ok
    }

    #[test]
    fn test_case_sensitivity() {
        let pattern = FuzzyConfig::builder()
            .search_term("Test")
            .case_sensitive(true)
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("Test"));
        assert!(!regex.is_match("test"));
        assert!(!regex.is_match("TEST"));
        assert!(!regex.is_match("testing"));
    }

    #[test]
    fn test_custom_config() {
        let pattern = FuzzyConfig::builder()
            .search_term("hello")
            .min_word_length(5)
            .required_char_ratio(0.8)
            .max_char_gap(2)
            .build()
            .build_pattern()
            .unwrap();
        println!("Generated pattern: {}", pattern);
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello"));
        assert!(regex.is_match("heello")); // small gap
        assert!(!regex.is_match("h e l l o")); // too big gaps

        // Debug prints for failing case
        println!("Testing 'h e l l o' against pattern");
        println!("Pattern matches: {}", regex.is_match("h e l l o"));
    }

    #[test]
    fn test_empty_pattern() {
        let result = FuzzyConfig::builder()
            .search_term("")
            .build()
            .build_pattern();
        assert!(matches!(result, Err(FuzzyError::EmptyPattern)));
    }

    #[test]
    fn test_whitespace_only_pattern() {
        let result = FuzzyConfig::builder()
            .search_term("   ")
            .build()
            .build_pattern();
        assert!(matches!(result, Err(FuzzyError::EmptyPattern)));
    }

    #[test]
    fn test_builder_methods() {
        let config = FuzzyConfig::builder()
            .search_term("test")
            .min_word_length(4)
            .required_char_ratio(0.75)
            .case_sensitive(true)
            .max_char_gap(3)
            .build();

        let pattern = config.build_pattern().unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("test"));
        assert!(!regex.is_match("TEST")); // case sensitive
    }

    #[test]
    fn test_special_characters() {
        let pattern = FuzzyConfig::builder()
            .search_term("hello.world$^")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello.world$^"));
        assert!(regex.is_match("hello world")); // Still matches without special chars
        assert!(regex.is_match("hello...world")); // Matches with extra dots
    }

    #[test]
    fn test_unicode_characters() {
        let pattern = FuzzyConfig::builder()
            .search_term("привет мир")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("привет мир"));
        assert!(regex.is_match("ПРИВЕТ МИР"));
        assert!(regex.is_match("привет добрый мир"));
    }

    #[test]
    fn test_extreme_char_gaps() {
        let pattern = FuzzyConfig::builder()
            .search_term("test")
            .max_char_gap(100)
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("t e s t"));
        assert!(regex.is_match("t....e....s....t"));

        // Test with minimum gap
        let pattern = FuzzyConfig::builder()
            .search_term("test")
            .max_char_gap(0)
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("test"));
        assert!(!regex.is_match("t e s t"));
    }

    #[test]
    fn test_extreme_word_lengths() {
        // Very short word
        let pattern = FuzzyConfig::builder()
            .search_term("a")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("a"));
        assert!(regex.is_match("abc"));

        // Very long word
        let long_word = "a".repeat(100);
        let pattern = FuzzyConfig::builder()
            .search_term(&long_word)
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match(&long_word));
        assert!(regex.is_match(&format!("{}b", &long_word)));
    }

    #[test]
    fn test_boundary_char_ratio() {
        // Test with 100% ratio
        let pattern = FuzzyConfig::builder()
            .search_term("test")
            .required_char_ratio(1.0)
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("test"));
        assert!(!regex.is_match("tes")); // Won't match with missing char

        // Test with minimum ratio
        let pattern = FuzzyConfig::builder()
            .search_term("test")
            .required_char_ratio(0.0)
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("test"));
        assert!(regex.is_match("t")); // Matches with just first char
    }

    #[test]
    fn test_multiple_spaces() {
        let pattern = FuzzyConfig::builder()
            .search_term("hello   world") // Multiple spaces
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("hello world"));
        assert!(regex.is_match("hello   world"));
        assert!(regex.is_match("hello \t\n world")); // Different whitespace
    }

    #[test]
    fn test_numbers_and_mixed_content() {
        let pattern = FuzzyConfig::builder()
            .search_term("test123 456")
            .build()
            .build_pattern()
            .unwrap();
        let regex = Regex::new(&pattern).unwrap();

        assert!(regex.is_match("test123 456"));
        assert!(regex.is_match("test 123 456"));
        assert!(regex.is_match("TEST123 456"));
    }

    #[test]
    fn test_logging() {
        use env_logger;
        let _ = env_logger::builder().is_test(true).try_init();

        let pattern = FuzzyConfig::builder()
            .search_term("test")
            .max_char_gap(2)
            .min_word_length(3)
            .required_char_ratio(0.8)
            .build()
            .build_pattern()
            .unwrap();

        let regex = Regex::new(&pattern).unwrap();
        assert!(regex.is_match("test"));

        // Test error logging
        let result = FuzzyConfig::builder()
            .search_term("")
            .build()
            .build_pattern();
        assert!(matches!(result, Err(FuzzyError::EmptyPattern)));
    }
}
