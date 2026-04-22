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
    assert!(string_capacity(&s) > 0);
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
    unsafe { memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 4) };
    assert_eq!(dest, src);
}

#[test]
fn test_memcopy_partial() {
    let src = [5u8, 6, 7, 8];
    let mut dest = [0u8; 4];
    println!("HEY");
    unsafe { memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 2) };
    assert_eq!(dest, [5, 6, 0, 0]);
}

#[test]
fn test_memcopy_zero() {
    let src = [1u8, 2, 3, 4];
    let mut dest = [0u8; 4];
    unsafe { memcopy(dest.as_mut_ptr(), src.as_ptr() as *mut u8, 0) };
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

#[test]
fn test_string_ptr_non_null() {
    let s = string_new();
    assert!(!string_ptr(&s).is_null());
}

#[test]
fn test_string_accomodate_extra_space_direct() {
    let mut s = string_new();
    let before = string_capacity(&s);
    string_accomodate_extra_space(&mut s, before + 5);
    assert!(string_capacity(&s) >= before + 5);
}

#[test]
fn test_string_from_str_direct() {
    let s = string_from_str("hello");
    assert!(string_eq(&s, &string_from_str("hello")));
}

fn parser_type_match(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::U8, Type::U8) => true,
        (Type::Usize, Type::Usize) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::Char, Type::Char) => true,
        (Type::Unit, Type::Unit) => true,
        (Type::Custom(a_name), Type::Custom(b_name)) => string_eq(a_name, b_name),
        _ => false,
    }
}

fn parser_types_match(a: &Types, b: &Types) -> bool {
    match (a, b) {
        (Types::Nil, Types::Nil) => true,
        (Types::Cons(a_head, a_tail), Types::Cons(b_head, b_tail)) => and(
            parser_type_match(a_head, b_head),
            parser_types_match(typesBox_deref(a_tail), typesBox_deref(b_tail)),
        ),
        _ => false,
    }
}

fn types_single(t: Type) -> Types {
    Types::Cons(t, typesBox_new(Types::Nil))
}

// ------------------------- Bool ----------------------------

#[test]
fn test_and() {
    assert_eq!(and(true, true), true);
    assert_eq!(and(true, false), false);
    assert_eq!(and(false, true), false);
    assert_eq!(and(false, false), false);
}

#[test]
fn test_or() {
    assert_eq!(or(true, true), true);
    assert_eq!(or(true, false), true);
    assert_eq!(or(false, true), true);
    assert_eq!(or(false, false), false);
}

#[test]
fn test_string_with_capacity() {
    let s = string_with_capacity(32);
    assert_eq!(string_len(&s), 0);
    assert_eq!(string_capacity(&s), 32);
}

#[test]
fn test_string_clone() {
    let mut s = string_new();
    string_push_str(&mut s, "clone me");
    let clone = string_clone(&s);
    assert!(string_eq(&s, &clone));
    assert!(string_ptr(&s) != string_ptr(&clone));
}

// ------------------------- Symbol Table ----------------------------

#[test]
fn test_global_symtable_prepend_and_contains() {
    let global = GlobalSymTable::Nil;
    assert!(!globalSymTable_contains(&global, &string_from_str("f")));

    let mut global = GlobalSymTable::Nil;
    let entry =
        SymTableEntry::Function(string_from_str("f"), FnSignature::Fn(Types::Nil, Type::U8));
    globalSymTable_prepend(&mut global, entry);
    match &global {
        GlobalSymTable::Cons(head, _) => {
            assert!(string_eq(symTableEntry_name(head), &string_from_str("f")));
        }
        GlobalSymTable::Nil => assert!(false, "expected non-empty global table"),
    }
}

#[test]
fn test_global_symtable_insert_function() {
    let mut global = GlobalSymTable::Nil;
    assert!(globalSymTable_insert_function(
        &mut global,
        string_from_str("f"),
        Types::Nil,
        Type::Usize
    ));
}

#[test]
fn test_global_symtable_insert_enum() {
    let mut global = GlobalSymTable::Nil;
    assert!(globalSymTable_insert_enum(
        &mut global,
        string_from_str("Color"),
        Types::Nil
    ));
}

#[test]
fn test_local_symtable_prepend_and_contains() {
    let local = LocalSymTable::Nil;
    assert!(!localSymTable_contains(&local, &string_from_str("x")));

    let mut local = LocalSymTable::Nil;
    let entry = SymTableEntry::Variable(string_from_str("x"), Type::Bool, true);
    localSymTable_prepend(&mut local, entry);
    match &local {
        LocalSymTable::Cons(head, _) => {
            assert!(string_eq(symTableEntry_name(head), &string_from_str("x")));
        }
        LocalSymTable::Nil => assert!(false, "expected non-empty local table"),
    }
}

#[test]
fn test_local_symtable_insert_variable() {
    let mut local = LocalSymTable::Nil;
    localSymTable_insert_variable(&mut local, string_from_str("x"), Type::U8, true);
    assert!(localSymTable_contains(&local, &string_from_str("x")));

    localSymTable_insert_variable(&mut local, string_from_str("x"), Type::U8, true);
    assert!(localSymTable_contains(&local, &string_from_str("x")));
}

#[test]
fn test_local_symtable_stack_push_pop_contains() {
    let mut stack = LocalSymTableStack::Nil;
    localSymTableStack_push(&mut stack);

    match &mut stack {
        LocalSymTableStack::Cons(local, _) => {
            localSymTable_insert_variable(local, string_from_str("x"), Type::Usize, false);
        }
        LocalSymTableStack::Nil => assert!(false, "expected non-empty stack"),
    }

    assert!(localSymTableStack_pop(&mut stack));
    assert!(!localSymTableStack_pop(&mut stack));
}

#[test]
fn test_symtable_new_and_contains() {
    let symtable = symTable_new();
    assert!(!symTable_contains(&symtable, &string_from_str("missing")));
}

#[test]
fn test_symtable_insert_function() {
    let mut symtable = symTable_new();
    assert!(symTable_insert_function(
        &mut symtable,
        string_from_str("f"),
        Types::Nil,
        Type::Usize
    ));
}

#[test]
fn test_symtable_insert_enum() {
    let mut symtable = symTable_new();
    assert!(symTable_insert_enum(
        &mut symtable,
        string_from_str("State"),
        Types::Nil
    ));
}

#[test]
fn test_symtable_scope_and_variables() {
    let mut symtable = symTable_new();
    symTable_insert_variable(&mut symtable, string_from_str("x"), Type::U8, true);
    assert!(!symTable_contains(&symtable, &string_from_str("x")));

    symTable_enter_scope(&mut symtable);
    symTable_insert_variable(&mut symtable, string_from_str("x"), Type::U8, true);
    assert!(symTable_contains(&symtable, &string_from_str("x")));

    symTable_enter_scope(&mut symtable);
    symTable_insert_variable(&mut symtable, string_from_str("x"), Type::Usize, false);
    assert!(symTable_contains(&symtable, &string_from_str("x")));
    assert!(symTable_leave_scope(&mut symtable));
    assert!(symTable_leave_scope(&mut symtable));
    assert!(!symTable_leave_scope(&mut symtable));
}

#[test]
fn test_symtable_entry_name() {
    let fn_entry = SymTableEntry::Function(
        string_from_str("f"),
        FnSignature::Fn(Types::Nil, Type::Unit),
    );
    assert!(string_eq(
        symTableEntry_name(&fn_entry),
        &string_from_str("f")
    ));

    let enum_entry = SymTableEntry::Enum(string_from_str("E"), Types::Nil);
    assert!(string_eq(
        symTableEntry_name(&enum_entry),
        &string_from_str("E")
    ));

    let var_entry = SymTableEntry::Variable(string_from_str("v"), Type::Char, true);
    assert!(string_eq(
        symTableEntry_name(&var_entry),
        &string_from_str("v")
    ));
}

#[test]
fn test_type_clone() {
    let custom = Type::Custom(string_from_str("MyType"));
    let cloned = type_clone(&custom);
    assert!(parser_type_match(&custom, &cloned));
}

#[test]
fn test_types_clone() {
    let types = types_single(Type::Custom(string_from_str("Node")));
    let cloned = types_clone(&types);
    assert!(parser_types_match(&types, &cloned));
}

#[test]
fn test_symtable_entry_clone() {
    let entry = SymTableEntry::Function(
        string_from_str("f"),
        FnSignature::Fn(types_single(Type::U8), Type::Custom(string_from_str("Ret"))),
    );
    let cloned = symTableEntry_clone(&entry);

    let name = symTableEntry_name(&cloned);
    assert!(string_eq(name, &string_from_str("f")));
}

#[test]
fn test_global_symtable_clone() {
    let global = GlobalSymTable::Nil;
    let cloned = globalSymTable_clone(&global);
    assert!(matches!(cloned, GlobalSymTable::Nil));
}

#[test]
fn test_local_symtable_clone() {
    let mut local = LocalSymTable::Nil;
    localSymTable_insert_variable(&mut local, string_from_str("x"), Type::U8, true);
    let cloned = localSymTable_clone(&local);
    match &cloned {
        LocalSymTable::Cons(head, _) => {
            assert!(string_eq(symTableEntry_name(head), &string_from_str("x")));
        }
        LocalSymTable::Nil => assert!(false, "expected non-empty local table"),
    }
}

#[test]
fn test_local_symtable_stack_clone() {
    let mut stack = LocalSymTableStack::Nil;
    localSymTableStack_push(&mut stack);
    match &mut stack {
        LocalSymTableStack::Cons(local, _) => {
            localSymTable_insert_variable(local, string_from_str("x"), Type::Usize, false);
        }
        LocalSymTableStack::Nil => assert!(false, "expected non-empty stack"),
    }

    let cloned = localSymTableStack_clone(&stack);
    match &cloned {
        LocalSymTableStack::Cons(local, _) => match local {
            LocalSymTable::Cons(head, _) => {
                assert!(string_eq(symTableEntry_name(head), &string_from_str("x")));
            }
            LocalSymTable::Nil => assert!(false, "expected non-empty local scope"),
        },
        LocalSymTableStack::Nil => assert!(false, "expected non-empty local stack"),
    }
}

#[test]
fn test_global_symtable_box_new_deref_clone() {
    let ptr = globalSymTableBox_new(GlobalSymTable::Nil);
    assert!(matches!(globalSymTableBox_deref(&ptr), GlobalSymTable::Nil));

    let cloned_ptr = globalSymTableBox_clone(&ptr);
    assert!(matches!(
        globalSymTableBox_deref(&cloned_ptr),
        GlobalSymTable::Nil
    ));
}

#[test]
fn test_local_symtable_box_new_deref_clone() {
    let ptr = localSymTableBox_new(LocalSymTable::Nil);
    assert!(matches!(localSymTableBox_deref(&ptr), LocalSymTable::Nil));

    let cloned_ptr = localSymTableBox_clone(&ptr);
    assert!(matches!(
        localSymTableBox_deref(&cloned_ptr),
        LocalSymTable::Nil
    ));
}

#[test]
fn test_local_symtable_stack_box_new_deref_clone() {
    let ptr = localSymTableStackBox_new(LocalSymTableStack::Nil);
    assert!(matches!(
        localSymTableStackBox_deref(&ptr),
        LocalSymTableStack::Nil
    ));

    let cloned_ptr = localSymTableStackBox_clone(&ptr);
    assert!(matches!(
        localSymTableStackBox_deref(&cloned_ptr),
        LocalSymTableStack::Nil
    ));
}

#[test]
fn test_types_box_new_deref_clone() {
    let ptr = typesBox_new(Types::Nil);
    assert!(matches!(typesBox_deref(&ptr), Types::Nil));

    let cloned_ptr = typesBox_clone(&ptr);
    assert!(matches!(typesBox_deref(&cloned_ptr), Types::Nil));
}
