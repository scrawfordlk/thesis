pub use std::string::String;

#[unsafe(no_mangle)]
pub fn print_i32(val: i32) {
    println!("{}", val);
}

#[unsafe(no_mangle)]
pub fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[unsafe(no_mangle)]
pub fn println(s: &str) {
    std::println!("{}", s);
}

#[unsafe(no_mangle)]
pub fn string_new() -> String {
    String::new()
}

#[unsafe(no_mangle)]
pub fn string_push_str(s: &mut String, text: &str) {
    s.push_str(text);
}

#[unsafe(no_mangle)]
pub fn string_as_str(s: &String) -> &str {
    s.as_str()
}
