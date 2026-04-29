fn test_llvm_unique_path(label: &str, extension: &str) -> std::path::PathBuf {
    let mut path: std::path::PathBuf = std::env::temp_dir();
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_nanos();
    path.push(format!(
        "thesis-llvm-{}-{}-{}.{}",
        label,
        std::process::id(),
        nanos,
        extension
    ));
    path
}

fn test_llvm_write_ir(label: &str, source: &str) -> std::path::PathBuf {
    let path: std::path::PathBuf = test_llvm_unique_path(label, "ll");
    std::fs::write(&path, source).expect("failed to write .ll test file");
    path
}

fn test_llvm_run_lli(path: &std::path::Path) -> i32 {
    let status: std::process::ExitStatus = std::process::Command::new("lli")
        .arg(path)
        .status()
        .expect("failed to execute lli");
    status.code().expect("lli terminated without exit code")
}

fn test_llvm_verify_with_llvm_as(path: &std::path::Path) {
    let bitcode_path: std::path::PathBuf = path.with_extension("bc");
    let status: std::process::ExitStatus = std::process::Command::new("llvm-as")
        .arg(path)
        .arg("-o")
        .arg(&bitcode_path)
        .status()
        .expect("failed to execute llvm-as");
    assert!(status.success(), "llvm-as rejected test IR");
}

#[test]
fn test_read_char_file_read_byte_and_read_all() {
    let path: std::path::PathBuf = test_llvm_write_ir("readchar", "ab");
    let path_str: &str = path.to_str().expect("utf8 path expected");

    let mut file: File = file_open(path_str);

    match file_read_byte(&mut file) {
        U8Option::Some(byte) => assert_eq!(byte, b'a'),
        U8Option::None => assert!(false, "expected first byte"),
    }

    match file_read_byte(&mut file) {
        U8Option::Some(byte) => assert_eq!(byte, b'b'),
        U8Option::None => assert!(false, "expected second byte"),
    }

    match file_read_byte(&mut file) {
        U8Option::Some(_) => assert!(false, "expected eof"),
        U8Option::None => {}
    }

    let all: String = file_read_all(path_str);
    assert!(string_eq(&all, &string_from_str("ab")));
}

#[test]
fn test_llvm_lexer_skips_comment_and_tokenizes_header() {
    let source: String = string_from_str("; comment\n\ndefine i64 @main() { ret i64 0 }");
    let mut lexer: LlvmLexer = llvmLexer_new(source);

    assert!(llvmToken_eq(
        llvmLexer_current_token(&lexer),
        &LlvmToken::Define
    ));

    llvmLexer_next_token(&mut lexer);
    assert!(llvmToken_eq(
        llvmLexer_current_token(&lexer),
        &LlvmToken::I64
    ));

    llvmLexer_next_token(&mut lexer);
    assert!(llvmToken_eq(
        llvmLexer_current_token(&lexer),
        &LlvmToken::At
    ));

    llvmLexer_next_token(&mut lexer);
    match llvmLexer_current_token(&lexer) {
        LlvmToken::Identifier(name) => assert!(string_eq(name, &string_from_str("main"))),
        _ => assert!(false, "expected main identifier"),
    }
}

#[test]
fn test_llvm_parse_and_emulate_constant_return() {
    let source: &str = "define i64 @main() { ret i64 42 }";
    let exit_code: usize = llvm_emulate_source_str(source);
    assert_eq!(exit_code, 42);
}

#[test]
fn test_llvm_parse_and_emulate_arithmetic() {
    let source: &str = "
define i64 @main() {
  %x = add i64 40, 2
  %y = mul i64 %x, 2
  %z = udiv i64 %y, 2
  ret i64 %z
}";
    let exit_code: usize = llvm_emulate_source_str(source);
    assert_eq!(exit_code, 42);
}

#[test]
fn test_llvm_parse_and_emulate_icmp_bool() {
    let source: &str = "
define i1 @main() {
  %cmp = icmp ult i64 7, 8
  ret i1 %cmp
}";
    let exit_code: usize = llvm_emulate_source_str(source);
    assert_eq!(exit_code, 1);
}

#[test]
fn test_llvm_parse_and_emulate_function_call() {
    let source: &str = "
define i64 @foo() {
  ret i64 7
}

define i64 @main() {
  %value = call i64 @foo()
  ret i64 %value
}";
    let exit_code: usize = llvm_emulate_source_str(source);
    assert_eq!(exit_code, 7);
}

#[test]
fn test_llvm_llvm_tools_accept_simple_program() {
    let path: std::path::PathBuf = std::path::PathBuf::from("tests/llvm_simple.ll");
    test_llvm_verify_with_llvm_as(&path);

    let lli_exit: i32 = test_llvm_run_lli(&path);
    let emulator_exit: usize = llvm_emulate_file(path.to_str().expect("utf8 path expected"));
    assert_eq!(llvm_exit_code_to_shell(emulator_exit), lli_exit);
}

#[test]
fn test_llvm_emulator_matches_lli_for_multiple_exit_codes() {
    let codes: [usize; 5] = [0, 1, 7, 42, 255];

    for code in codes {
        let source: std::string::String = format!("define i64 @main() {{ ret i64 {} }}", code);
        let path: std::path::PathBuf = test_llvm_write_ir("exit-codes", &source);
        test_llvm_verify_with_llvm_as(&path);

        let lli_exit: i32 = test_llvm_run_lli(&path);
        let emulated_exit: usize =
            llvm_emulate_file(path.to_str().expect("utf8 path expected"));
        let emulated_shell_exit: i32 = llvm_exit_code_to_shell(emulated_exit);

        assert_eq!(emulated_shell_exit, lli_exit);
    }
}
