# 🔍 Fuzzy Search

> A powerful and flexible fuzzy search library for Rust, designed for intelligent pattern matching and search capabilities.

[![Crates.io](https://img.shields.io/crates/v/fuzzy_search.svg)](https://crates.io/crates/fuzzy_search)
[![Documentation](https://docs.rs/fuzzy_search/badge.svg)](https://docs.rs/fuzzy_search)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- 🎯 **Intelligent Pattern Matching**: Automatically generates regex patterns for fuzzy searching
- 🛠️ **Highly Configurable**: Customize search behavior with flexible options
- 🌈 **Unicode Support**: Full support for special characters and Unicode
- 📊 **Smart Gap Handling**: Configurable character gap limits for precise matching
- 🔄 **Case Sensitivity**: Optional case-sensitive or case-insensitive matching
- 📝 **Detailed Logging**: Comprehensive logging for debugging and monitoring
- ⚡ **Performance**: Optimized pattern generation for efficient searching

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fuzzy_search = "0.1.0"
```

## 🚀 Quick Start

```rust
use fuzzy_search::{FuzzyConfig, fuzzy_search_pattern};

// Simple usage with default settings
let pattern = fuzzy_search_pattern("hello");
assert!(pattern.is_match("hello"));
assert!(pattern.is_match("HELLO"));

// Advanced configuration
let pattern = FuzzyConfig::builder()
    .search_term("hello")
    .max_char_gap(2)        // Allow up to 2 characters between matches
    .min_word_length(3)     // Minimum word length to match
    .required_char_ratio(0.8) // Require 80% of characters to match
    .case_sensitive(false)  // Case-insensitive matching
    .build()
    .build_pattern()
    .unwrap();

// Pattern will match:
assert!(pattern.is_match("hello"));     // Exact match
assert!(pattern.is_match("heello"));    // Small gap
assert!(!pattern.is_match("h e l l o")); // Too many gaps
```

## 🎨 Configuration Options

| Option                | Description                                | Default |
| --------------------- | ------------------------------------------ | ------- |
| `max_char_gap`        | Maximum characters allowed between matches | 2       |
| `min_word_length`     | Minimum length of words to match           | 3       |
| `required_char_ratio` | Required ratio of matching characters      | 0.8     |
| `case_sensitive`      | Enable case-sensitive matching             | false   |

## 🔍 Pattern Generation Rules

The library uses smart pattern generation with different strategies based on gap size:

- **Large Gaps** (>10 characters):

  ```
  Allows any characters including spaces
  Example: "h....e....l....l....o"
  ```

- **Small Gaps** (1-10 characters):

  ```
  Only allows non-space characters
  Example: "heello" but not "h e l l o"
  ```

- **No Gaps** (0 characters):
  ```
  Requires exact character sequence
  Example: "hello" only
  ```

## 📊 Logging

The library uses the `log` crate for detailed insights:

```rust
// Enable logging in your application
env_logger::init();

// Debug logs show pattern generation details
let pattern = FuzzyConfig::builder()
    .search_term("hello")
    .build()
    .build_pattern()
    .unwrap();

// Logs will show:
// DEBUG: Creating pattern for word: hello
// DEBUG: Using small gap pattern with max_char_gap=2
// DEBUG: Final word pattern: (?:[hH][^\s]{0,2}[eE]...)
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
cargo test
```

## 📈 Performance Considerations

- Pattern generation is optimized for both small and large search terms
- Regex compilation is cached where possible
- Smart gap handling reduces backtracking in regex engine

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Credits

Created with ❤️ by the Rust community. Special thanks to all contributors!

---

<div align="center">
Made with 🦀 Rust and ❤️ for the community
</div>
