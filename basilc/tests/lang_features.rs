use basil_parser::parse;
use basil_compiler::compile;
use basil_vm::VM;
use basil_bytecode::Value;

fn run(src: &str) -> (Vec<String>, Vec<Value>) {
    let ast = parse(src).expect("parse");
    let bc = compile(&ast).expect("compile");
    let mut vm = VM::new(bc);
    vm.run().expect("vm run");
    vm.globals_snapshot()
}

fn get_global_idx(names: &[String], name: &str) -> Option<usize> {
    names.iter().position(|n| n == name)
}

#[test]
fn const_declarations_and_values() {
    let src = r#"
CONST DEFAULT_OS = "L"
CONST MAX_RETRIES = 3
CONST PI = 3.14159
"#;
    let (names, vals) = run(src);
    // Strings
    match &vals[get_global_idx(&names, "DEFAULT_OS").expect("DEFAULT_OS")] {
        Value::Str(s) => assert_eq!(s, "L"),
        other => panic!("expected string, got {:?}", other),
    }
    // Numeric constants are represented as numbers (float) in Basil by default
    match &vals[get_global_idx(&names, "MAX_RETRIES").expect("MAX_RETRIES")] {
        Value::Num(n) => assert_eq!(*n, 3.0),
        Value::Int(i) => assert_eq!(*i, 3),
        other => panic!("expected numeric, got {:?}", other),
    }
    match &vals[get_global_idx(&names, "PI").expect("PI")] {
        Value::Num(n) => assert!((*n - 3.14159).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn const_reassignment_is_error() {
    let src = r#"
CONST LIMIT = 3
LIMIT = 4
"#;
    let ast = parse(src).expect("parse");
    let err = compile(&ast).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.to_ascii_lowercase().contains("cannot assign to constant"), "unexpected error: {}", msg);
}

#[test]
fn dim_multiple_defaults_and_single() {
    let src = r#"
DIM a$, b$, c$
DIM i%, j%
DIM x, y
"#;
    let (names, vals) = run(src);
    // strings default to empty
    for n in ["a$", "b$", "c$"] { match &vals[get_global_idx(&names, n).expect(n)] { Value::Str(s) => assert!(s.is_empty()), other => panic!("{} expected string, got {:?}", n, other) } }
    // integers default to 0
    for n in ["i%", "j%"] { match &vals[get_global_idx(&names, n).expect(n)] { Value::Int(i) => assert_eq!(*i, 0), Value::Num(nv) => assert_eq!(*nv, 0.0), other => panic!("{} expected numeric 0, got {:?}", n, other) } }
    // unsuffixed numeric defaults to 0.0
    for n in ["x", "y"] { match &vals[get_global_idx(&names, n).expect(n)] { Value::Num(nv) => assert_eq!(*nv, 0.0), Value::Int(i) => assert_eq!(*i, 0), other => panic!("{} expected numeric 0, got {:?}", n, other) } }
}

#[test]
fn implicit_let_assignment_scalar_and_index() {
    // scalar implicit LET
    let src1 = "DIM x%\nx% = 5\n";
    let (names1, vals1) = run(src1);
    match &vals1[get_global_idx(&names1, "x%").expect("x%")] {
        Value::Int(i) => assert_eq!(*i, 5),
        Value::Num(n) => assert_eq!(*n as i64, 5),
        other => panic!("expected int, got {:?}", other),
    }

    // array element implicit LET: DIM arr(3): arr(2) = 7
    let src2 = "DIM arr%(3)\narr%(2) = 7\n";
    let (names2, vals2) = run(src2);
    // We don't have a direct getter for array element here; ensure the global exists and is an array by Describe? For now, just ensure name exists.
    assert!(get_global_idx(&names2, "arr%").is_some());
}
