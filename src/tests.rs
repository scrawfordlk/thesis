// Tests for main.rs
// Note: Tests are written in full Rust (unlike the code in main.rs, which
// is written in the restricted subset of Rust).

mod tests {
    #[allow(unused_imports)]
    use super::{
        CharOption, GlobalSymTable, GlobalSymTableBox, Lexer, LocalSymTable, LocalSymTableBox,
        LocalSymTableStack, LocalSymTableStackBox, SourceFile, SourceLocation, String, SymTable,
        SymTableEntry, Token, Type, Types, TypesBox, alloc, and, global_symtable_box_clone,
        global_symtable_box_deref, global_symtable_box_new, global_symtable_clone,
        global_symtable_contains, global_symtable_insert_enum, global_symtable_insert_function,
        global_symtable_prepend, identifier_to_token, is_alpha, is_alphanumeric, is_digit,
        is_whitespace, lexer_consume_char, lexer_error, lexer_expect_char, lexer_location,
        lexer_peek_char, lexer_sourcefile, local_symtable_box_clone, local_symtable_box_deref,
        local_symtable_box_new, local_symtable_clone, local_symtable_contains,
        local_symtable_insert_variable, local_symtable_prepend, local_symtable_stack_box_clone,
        local_symtable_stack_box_deref, local_symtable_stack_box_new, local_symtable_stack_clone,
        local_symtable_stack_contains, local_symtable_stack_pop, local_symtable_stack_push,
        memcopy, next_token, or, parse_to_llvm, ptr_add, scan_bang, scan_char_literal, scan_colon,
        scan_equals, scan_escape_char, scan_greater, scan_identifier, scan_integer, scan_less,
        scan_minus, scan_slash, scan_string_literal, scan_symbol, skip_line_comment,
        skip_whitespace, string_accomodate_extra_space, string_capacity, string_clone, string_eq,
        string_from_str, string_get, string_len, string_new, string_ptr, string_push,
        string_push_str, string_with_capacity, symtable_contains, symtable_enter_scope,
        symtable_entry_clone, symtable_entry_name, symtable_insert_enum, symtable_insert_function,
        symtable_insert_variable, symtable_leave_scope, symtable_new, token_eq, type_clone,
        types_box_clone, types_box_deref, types_box_new, types_clone, unwrap_char,
    };

    // Helper to convert our String to std::string::String for easy comparison
    fn to_std_string(s: &String) -> std::string::String {
        (0..string_len(s))
            .map(|i| unwrap_char(string_get(s, i)))
            .collect()
    }

    include!("test_core.rs");
    include!("test_lexer.rs");
    include!("test_parser.rs");
}
