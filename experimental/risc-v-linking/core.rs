use wrapper::{String, add, print_i32, println, string_as_str, string_new, string_push_str};

fn main() {
    let result: i32 = add(10, 32);
    print_i32(result);

    let mut s: String = string_new();
    string_push_str(&mut s, "Hello from SRS core on RISC-V!");
    let text: &str = string_as_str(&s);
    println(text);
}
