fn main() -> usize {
    let a: usize = 2;
    let b: usize = {
        let a: u8 = 5 as u8;
        let b: u8 = 16 as u8;
        (a + b) as usize
    };
    let c: usize = a * b;
    let a: usize = c + a;
    let result: usize = a;
    result
}
