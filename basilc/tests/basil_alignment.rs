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
fn triple_quoted_strings() {
    let src = r#"
        LET s$ = """Line 1
Line 2"""
    "#;
    let (names, vals) = run(src);
    match &vals[get_global_idx(&names, "s$").expect("s$")] {
        Value::Str(s) => assert_eq!(s, "Line 1\nLine 2"),
        other => panic!("expected string, got {:?}", other),
    }
}

#[test]
fn unicode_identifiers() {
    let src = "LET 变量 = 123";
    let (names, vals) = run(src);
    match &vals[get_global_idx(&names, "变量").expect("变量")] {
        Value::Int(i) => assert_eq!(*i, 123),
        Value::Num(n) => assert_eq!(*n, 123.0),
        other => panic!("expected int/num, got {:?}", other),
    }
}

#[test]
fn split_returns_list() {
    let src = r#"LET l = SPLIT$("a,b,c", ",")"#;
    let (names, vals) = run(src);
    match &vals[get_global_idx(&names, "l").expect("l")] {
        Value::List(items) => {
            let v = items.borrow();
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::Str("a".to_string()));
            assert_eq!(v[1], Value::Str("b".to_string()));
            assert_eq!(v[2], Value::Str("c".to_string()));
        }
        other => panic!("expected list, got {:?}", other),
    }
}

#[test]
fn val_with_bases() {
    let src = r#"
        LET h = VAL("&HFF")
        LET b = VAL("&B101")
        LET o = VAL("&O77")
        LET i = VAL("123")
    "#;
    let (names, vals) = run(src);
    assert_eq!(vals[get_global_idx(&names, "h").unwrap()], Value::Int(255));
    assert_eq!(vals[get_global_idx(&names, "b").unwrap()], Value::Int(5));
    assert_eq!(vals[get_global_idx(&names, "o").unwrap()], Value::Int(63));
    assert_eq!(vals[get_global_idx(&names, "i").unwrap()], Value::Int(123));
}

#[test]
fn str_refinement() {
    let src = r#"
        LET bt$ = STR$(TRUE)
        LET bf$ = STR$(FALSE)
        LET n1$ = STR$(123.45)
        LET n2$ = STR$(100.0)
    "#;
    let (names, vals) = run(src);
    assert_eq!(vals[get_global_idx(&names, "bt$").unwrap()], Value::Str("TRUE".into()));
    assert_eq!(vals[get_global_idx(&names, "bf$").unwrap()], Value::Str("FALSE".into()));
    assert_eq!(vals[get_global_idx(&names, "n1$").unwrap()], Value::Str("123.45".into()));
    assert_eq!(vals[get_global_idx(&names, "n2$").unwrap()], Value::Str("100".into()));
}

#[test]
fn render_function() {
    let src = r#"
        LET ctx = { "name": "World", "val": 42 }
        LET out$ = RENDER$("Hello \#{name}, the value is \#{val}.", ctx)
    "#;
    let (names, vals) = run(src);
    assert_eq!(vals[get_global_idx(&names, "out$").unwrap()], Value::Str("Hello World, the value is 42.".into()));
}

#[test]
fn vm_safety_gosub_base() {
    // In a function, calling RETURN without GOSUB should error even if there was a GOSUB outside
    let src = "
GOSUB outer
STOP

outer:
  myfunc()
  RETURN

FUNC myfunc()
  RETURN
ENDFUNC
";
    let ast = parse(src).expect("parse");
    let bc = compile(&ast).expect("compile");
    let mut vm = VM::new(bc);
    let res = vm.run();
    assert!(res.is_err());
    let msg = format!("{}", res.unwrap_err());
    assert!(msg.contains("RETURN without GOSUB"));
}
