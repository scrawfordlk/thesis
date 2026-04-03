// Tests for main.rs
// Note: Tests are written in full Rust (unlike the code in main.rs, which
// is written in the restricted subset of Rust).

mod tests {
    use super::*;

    // Helper to convert our String to std::string::String for easy comparison
    fn to_std_string(s: &String) -> std::string::String {
        (0..string_len(s))
            .map(|i| unwrap_char(string_get(s, i)))
            .collect()
    }

    // ----------------------- CharOption --------------------------

    #[test]
    fn test_unwrap_char_some() {
        assert_eq!(unwrap_char(CharOption::Some('a')), 'a');
    }

    #[test]
    #[should_panic(expected = "unwrap failed")]
    fn test_unwrap_char_none() {
        unwrap_char(CharOption::None);
    }

    // ------------------------- String ----------------------------

    #[test]
    fn test_string_new() {
        let s = string_new();
        assert_eq!(string_len(&s), 0);
        assert_eq!(string_capacity(&s), 1);
    }

    #[test]
    fn test_string_push() {
        let mut s = string_new();
        string_push(&mut s, 'H');
        assert_eq!(string_len(&s), 1);
        assert_eq!(to_std_string(&s), "H");
    }

    #[test]
    fn test_string_push_multiple() {
        let mut s = string_new();
        for c in ['a', 'b', 'c'] {
            string_push(&mut s, c);
        }
        assert_eq!(string_len(&s), 3);
        assert_eq!(to_std_string(&s), "abc");
    }

    #[test]
    fn test_string_push_str() {
        let mut s = string_new();
        string_push_str(&mut s, "Hello");
        assert_eq!(string_len(&s), 5);
        assert_eq!(to_std_string(&s), "Hello");
    }

    #[test]
    fn test_string_push_str_empty() {
        let mut s = string_new();
        string_push_str(&mut s, "");
        assert_eq!(string_len(&s), 0);
        assert_eq!(to_std_string(&s), "");
    }

    #[test]
    fn test_string_push_and_push_str_combined() {
        let mut s = string_new();
        string_push_str(&mut s, "Hi");
        string_push(&mut s, '!');
        assert_eq!(to_std_string(&s), "Hi!");
    }

    #[test]
    fn test_string_get_out_of_bounds() {
        let s = string_new();
        assert!(matches!(string_get(&s, 0), CharOption::None));
    }

    #[test]
    fn test_string_get_out_of_bounds_nonempty() {
        let mut s = string_new();
        string_push(&mut s, 'x');
        assert!(matches!(string_get(&s, 1), CharOption::None));
    }

    #[test]
    fn test_string_capacity_grows() {
        let mut s = string_new();
        let initial_cap = string_capacity(&s);
        for _ in 0..(initial_cap + 5) {
            string_push(&mut s, 'x');
        }
        assert!(string_capacity(&s) > initial_cap);
        assert_eq!(string_len(&s), initial_cap + 5);
    }

    // ------------------------- Memory ----------------------------

    #[test]
    fn test_ptr_add() {
        let data: [u8; 4] = [10, 20, 30, 40];
        let ptr = data.as_ptr() as *mut u8;
        unsafe {
            for (i, &expected) in data.iter().enumerate() {
                assert_eq!(*ptr_add(ptr, i), expected);
            }
        }
    }

    #[test]
    fn test_memcopy() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0u8; 4];
        memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 4);
        assert_eq!(dest, src);
    }

    #[test]
    fn test_memcopy_partial() {
        let src = [5u8, 6, 7, 8];
        let mut dest = [0u8; 4];
        memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 2);
        assert_eq!(dest, [5, 6, 0, 0]);
    }

    #[test]
    fn test_memcopy_zero() {
        let src = [1u8, 2, 3, 4];
        let mut dest = [0u8; 4];
        memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 0);
        assert_eq!(dest, [0; 4]);
    }

    #[test]
    fn test_alloc() {
        let ptr = alloc(16, 1);
        assert!(!ptr.is_null());
        // Verify zeroed allocation
        unsafe {
            for i in 0..16 {
                assert_eq!(*ptr_add(ptr, i), 0);
            }
        }
    }
}
