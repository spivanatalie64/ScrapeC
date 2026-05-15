// tests/test_match.rs

use scrapec::lexer::lex;
use scrapec::parser::parse;
use scrapec::types::{infer_type, Type, TypeEnv};
use scrapec::codegen::generate_c;

#[test]
fn test_match_expression() {
    let source = r#"
        let opt: Option<int> = Some(10);
        let result = match opt {
            Some(x) => x,
            None => 0,
        };
    "#;

    let tokens = lex(source);
    let ast = parse(&tokens);

    let mut env = TypeEnv::new();
    for node in &ast {
        infer_type(node, &mut env);
    }
    
    // Test type inference
    assert_eq!(env.get("opt").unwrap(), &Type::Option(Box::new(Type::Int)));
    assert_eq!(env.get("result").unwrap(), &Type::Int);

    // Test C code generation
    let c_code = generate_c(&ast, &env);
    
    // Check for variable declarations and the match expression
    assert!(c_code.contains("Option_i64 opt = some_i64(10);"));
    assert!(c_code.contains("int64_t result = ({if (opt.is_some) {"));
    assert!(c_code.contains("int64_t x = opt.value;"));
    assert!(c_code.contains("(x);"));
    assert!(c_code.contains("} else if (!opt.is_some) {"));
    assert!(c_code.contains("(0);"));
}
