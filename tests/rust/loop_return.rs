fn main() -> usize {
    let mut i: usize = 0;
    while i < 100 {
        while i == 42 {
            return i;
        }

        i = i + 1;
    }
    return 0;
}
