// tests/test_type_inference.rs

use scrapec::lexer::lex;
use scrapec::parser::parse;
use scrapec::types::{infer_type, Type, TypeEnv};

#[test]
fn test_let_statement_type_inference() {
    let source = r#"
        let an_integer = 10;
        let a_float = 3.14;
        let a_boolean = true;
    "#;

    let tokens = lex(source);
    let ast = parse(&tokens);

    let mut env = TypeEnv::new();
    for node in ast {
        infer_type(&node, &mut env);
    }

    assert_eq!(env.get("an_integer").unwrap(), &Type::Int);
    assert_eq!(env.get("a_float").unwrap(), &Type::Float);
    assert_eq!(env.get("a_boolean").unwrap(), &Type::Bool);
}

#[test]
fn test_binary_op_type_inference() {
    let source = r#"
        let x = 10 + 5;
        let y = 3.14 * 2.0;
    "#;

    let tokens = lex(source);
    let ast = parse(&tokens);

    let mut env = TypeEnv::new();
    for node in ast {
        infer_type(&node, &mut env);
    }

    assert_eq!(env.get("x").unwrap(), &Type::Int);
    assert_eq!(env.get("y").unwrap(), &Type::Float);
}

#[test]
#[should_panic]
fn test_binary_op_type_mismatch() {
    let source = "let z = 10 + true;";
    let tokens = lex(source);
    let ast = parse(&tokens);
    let mut env = TypeEnv::new();
    for node in ast {
        infer_type(&node, &mut env);
    }
}
