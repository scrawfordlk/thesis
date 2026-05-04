// Tests for main.rs
// Note: Tests are written in full Rust (unlike the code in main.rs, which
// is written in the restricted subset of Rust).

mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Helper to convert our String to std::string::String for easy comparison
    fn to_std_string(s: &String) -> std::string::String {
        (0..string_len(s))
            .map(|i| unwrap::<char>(string_get(s, i)))
            .collect::<std::string::String>()
    }

    include!("test_library.rs");
    include!("test_lexer.rs");
}
