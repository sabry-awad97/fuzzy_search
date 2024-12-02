# Fuzzy Search 🔍

A lightweight and efficient Rust library for fuzzy text searching with regex pattern generation. This library provides flexible pattern matching that's tolerant to typos and variations in text.

## Features ✨

- Case-insensitive matching
- Multi-word search support
- Typo-tolerant pattern generation
- Flexible character spacing
- Special handling for short words
- Comprehensive test coverage

## Installation 📦

Add this to your `Cargo.toml`:

```toml
[dependencies]
fuzzy_search = "0.1.0"
regex = "1.9.0"
```

## Usage 🚀

### Basic Usage

```rust
use fuzzy_search::fuzzy_search_pattern;
use regex::Regex;

fn main() {
    // Quick pattern generation with default settings
    let pattern = fuzzy_search_pattern("hello world");
    let regex = Regex::new(&pattern).unwrap();

    assert!(regex.is_match("hello world"));     // Exact match
    assert!(regex.is_match("HELLO WORLD"));     // Case insensitive
    assert!(regex.is_match("hello there world")); // Words with content between
}
```

### Advanced Usage with Builder Pattern

```rust
use fuzzy_search::FuzzySearchBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom fuzzy search pattern
    let regex = FuzzySearchBuilder::new("hello world")
        .min_word_length(4)           // Custom minimum word length
        .required_char_ratio(0.7)     // Require 70% of characters to match
        .case_sensitive(true)         // Enable case sensitivity
        .max_char_gap(5)             // Maximum gap between characters
        .compile()?;                  // Build and compile the regex

    assert!(regex.is_match("Hello World"));
    assert!(!regex.is_match("hello world")); // Won't match due to case sensitivity
    Ok(())
}
```

## Configuration Options ⚙️

The `FuzzySearchBuilder` provides several configuration options:

- `min_word_length`: Minimum word length for applying typo tolerance (default: 3)
- `required_char_ratio`: Required character ratio for longer words (default: 0.5)
- `case_sensitive`: Enable/disable case sensitivity (default: false)
- `max_char_gap`: Maximum allowed character gap (default: 10)

## Error Handling 🛡️

The library provides a custom error type `FuzzyError` for proper error handling:

```rust
pub enum FuzzyError {
    InvalidPattern(String),
    RegexError(regex::Error),
}
```

## Performance Considerations 🚀

- The builder pattern allows for pattern reuse
- Compiled regex patterns can be cached for repeated use
- Configurable character gap limits help control matching performance

## How It Works 🛠️

The library generates regex patterns that:

1. Split search terms into individual words
2. Create flexible patterns that match characters in sequence
3. Allow for typos and variations in longer words
4. Handle short words (≤3 characters) with special consideration
5. Maintain case insensitivity throughout

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request.

## License 📄

MIT License
