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

```rust
use fuzzy_search::fuzzy_search_pattern;
use regex::Regex;

fn main() {
    // Generate a fuzzy search pattern
    let pattern = fuzzy_search_pattern("hello world");
    let regex = Regex::new(&pattern).unwrap();

    // Test various matches
    assert!(regex.is_match("hello world"));     // Exact match
    assert!(regex.is_match("HELLO WORLD"));     // Case insensitive
    assert!(regex.is_match("hello there world")); // Words with content between
}
```

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
