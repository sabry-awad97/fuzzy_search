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

### Advanced Usage with Typed Builder

```rust
use fuzzy_search::FuzzyConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom fuzzy search pattern with compile-time validation
    let regex = FuzzyConfig::builder()
        .search_term("hello world")    // Required field
        .min_word_length(4)           // Optional with default = 3
        .required_char_ratio(0.7)     // Optional with default = 0.5
        .case_sensitive(true)         // Optional with default = false
        .max_char_gap(5)             // Optional with default = 10
        .build()                     // Build the config
        .compile()?;                 // Compile into regex

    assert!(regex.is_match("Hello World"));
    assert!(!regex.is_match("hello world")); // Won't match due to case sensitivity
    Ok(())
}
```

## Configuration Options ⚙️

The `FuzzyConfig` builder provides several configuration options with compile-time validation:

- `search_term`: The search term to create pattern for (required)
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

- Compile-time validation prevents runtime errors
- Type-safe builder pattern ensures correct configuration
- Configurable character gap limits help control matching performance
- Compiled regex patterns can be cached for repeated use

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
