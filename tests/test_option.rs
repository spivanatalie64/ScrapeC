// tests/test_option.rs

use scrapec::lexer::lex;
use scrapec::parser::parse;
use scrapec::types::{infer_type, Type, TypeEnv};
use scrapec::codegen::generate_c;

#[test]
fn test_option_type_inference_and_codegen() {
    let source = r#"
        let x: Option<int> = Some(10);
        let y: Option<int> = None;
    "#;

    let tokens = lex(source);
    let ast = parse(&tokens);

    let mut env = TypeEnv::new();
    for node in &ast {
        infer_type(node, &mut env);
    }
    
    // Test type inference
    let expected_type = Type::Option(Box::new(Type::Int));
    assert_eq!(env.get("x").unwrap(), &expected_type);
    assert_eq!(env.get("y").unwrap(), &expected_type);

    // Test C code generation
    let c_code = generate_c(&ast, &env);

    // Check for struct definition
    assert!(c_code.contains("typedef struct Option_i64"));
    assert!(c_code.contains("bool is_some;"));
    assert!(c_code.contains("int64_t value;"));
    
    // Check for helper functions
    assert!(c_code.contains("Option_i64 some_i64(int64_t val)"));
    assert!(c_code.contains("Option_i64 none_i64()"));

    // Check for variable declarations
    assert!(c_code.contains("Option_i64 x = some_i64(10);"));
    // The `None` case is tricky, the generator currently hardcodes `none_i64()`.
    // This will need improvement, but for now, we test the current behavior.
    assert!(c_code.contains("Option_i64 y = none_i64();"));
}
