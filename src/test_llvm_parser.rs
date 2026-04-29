fn llvm_test_lli_available() -> bool {
    match std::process::Command::new("lli").arg("--version").status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn llvm_test_read_fixture(path: &str) -> std::string::String {
    std::fs::read_to_string(path).expect("failed to read LLVM fixture")
}

fn llvm_test_run_lli(path: &str) -> i32 {
    let status: std::process::ExitStatus = std::process::Command::new("lli")
        .arg(path)
        .status()
        .expect("failed to run lli");
    status.code().expect("lli terminated without exit code")
}

fn llvm_test_block_count(blocks: &InstructionBlockList) -> usize {
    match blocks {
        InstructionBlockList::Nil => 0,
        InstructionBlockList::Cons(_, tail) => {
            1 + llvm_test_block_count(instructionBlockListBox_deref(tail))
        }
    }
}

#[test]
fn test_llvm_parse_simple_implicit_entry_block() {
    let source: &str = "
define i64 @main() {
  %x = add i64 40, 2
  ret i64 %x
}";
    let symtable: LlvmSymTable = llvmParser_parse_to_ast(source);
    let function_name: String = string_from_str("main");
    let function: &LlvmFunction = llvmSymTable_lookup_function(&symtable, &function_name);
    match function {
        LlvmFunction::Function(return_type, _, blocks) => {
            assert!(llvmType_eq(return_type, &LlvmType::I64));
            assert_eq!(llvm_test_block_count(blocks), 1);

            match blocks {
                InstructionBlockList::Cons(block, _) => match block {
                    InstructionBlock::Block(label, instructions) => {
                        assert!(string_eq(label, &string_from_str("entry")));
                        match instructions {
                            InstructionList::Cons(first, tail) => {
                                match first {
                                    Instruction::Assignment(AssignInstruction::Assign(
                                        name,
                                        AssignOp::Binary(
                                            BinaryOp::Add,
                                            ty,
                                            LlvmValue::Literal(40),
                                            LlvmValue::Literal(2),
                                        ),
                                    )) => {
                                        assert!(string_eq(name, &string_from_str("x")));
                                        assert!(llvmType_eq(ty, &LlvmType::I64));
                                    }
                                    _ => assert!(false, "unexpected first instruction"),
                                }
                                match instructionListBox_deref(tail) {
                                    InstructionList::Cons(second, _) => match second {
                                        Instruction::Terminator(TerminatorInstruction::Ret(
                                            ret_ty,
                                            LlvmValue::Register(name),
                                        )) => {
                                            assert!(llvmType_eq(ret_ty, &LlvmType::I64));
                                            assert!(string_eq(name, &string_from_str("x")));
                                        }
                                        _ => assert!(false, "unexpected return instruction"),
                                    },
                                    _ => assert!(false, "missing return instruction"),
                                }
                            }
                            _ => assert!(false, "missing assignment instruction"),
                        }
                    }
                },
                InstructionBlockList::Nil => assert!(false, "expected one block"),
            }
        }
    }
}

#[test]
fn test_llvm_parse_global_string_and_gep() {
    let source: std::string::String = llvm_test_read_fixture("tests/llvm_parse_gep.ll");
    let symtable: LlvmSymTable = llvmParser_parse_to_ast(&source);

    let global_name: String = string_from_str(".msg");
    let value: &String = llvmSymTable_lookup_string(&symtable, &global_name);
    assert!(string_eq(value, &string_from_str("hello\0")));
}

#[test]
fn test_llvm_emulator_matches_lli_simple_arithmetic() {
    if !llvm_test_lli_available() {
        return;
    }

    let fixture_path: &str = "tests/llvm_emu_simple.ll";
    let source: std::string::String = llvm_test_read_fixture(fixture_path);

    let lli_exit: i32 = llvm_test_run_lli(fixture_path);
    let emulated_raw: usize = llvm_emulate_source_str(&source);
    let emulated_shell: i32 = llvm_exit_code_to_shell(emulated_raw);

    assert_eq!(emulated_shell, lli_exit);
}

#[test]
fn test_llvm_emulator_matches_lli_branching() {
    if !llvm_test_lli_available() {
        return;
    }

    let fixture_path: &str = "tests/llvm_emu_branch.ll";
    let source: std::string::String = llvm_test_read_fixture(fixture_path);

    let lli_exit: i32 = llvm_test_run_lli(fixture_path);
    let emulated_raw: usize = llvm_emulate_source_str(&source);
    let emulated_shell: i32 = llvm_exit_code_to_shell(emulated_raw);

    assert_eq!(emulated_shell, lli_exit);
}

#[test]
fn test_llvm_emulator_matches_lli_function_call() {
    if !llvm_test_lli_available() {
        return;
    }

    let fixture_path: &str = "tests/llvm_emu_call.ll";
    let source: std::string::String = llvm_test_read_fixture(fixture_path);

    let lli_exit: i32 = llvm_test_run_lli(fixture_path);
    let emulated_raw: usize = llvm_emulate_source_str(&source);
    let emulated_shell: i32 = llvm_exit_code_to_shell(emulated_raw);

    assert_eq!(emulated_shell, lli_exit);
}
