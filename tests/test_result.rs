// tests/test_result.rs

use scrapec::lexer::lex;
use scrapec::parser::parse;
use scrapec::types::{infer_type, Type, TypeEnv};
use scrapec::codegen::generate_c;

#[test]
fn test_result_type_inference_and_codegen() {
    let source = r#"
        let x: Result<int, MyError> = Ok(10);
        let y: Result<int, MyError> = Err(my_error_instance);
    "#;

    let tokens = lex(source);
    let ast = parse(&tokens);

    let mut env = TypeEnv::new();
    
    // Manually add the custom error type and a variable of that type to the environment
    let my_error_type = Type::Custom("MyError".to_string());
    env.insert("MyError".to_string(), my_error_type.clone());
    env.insert("my_error_instance".to_string(), my_error_type.clone());


    for node in &ast {
        infer_type(node, &mut env);
    }
    
    // Test type inference
    let expected_type = Type::Result(
        Box::new(Type::Int), 
        Box::new(my_error_type)
    );
    assert_eq!(env.get("x").unwrap(), &expected_type);
    assert_eq!(env.get("y").unwrap(), &expected_type);

    // Test C code generation
    let c_code = generate_c(&ast, &env);
    
    // Check for custom type definition
    assert!(c_code.contains("typedef struct MyError { int dummy; } MyError;"));

    // Check for struct definition
    assert!(c_code.contains("typedef struct Result_i64_MyError"));
    assert!(c_code.contains("bool is_ok;"));
    assert!(c_code.contains("union {"));
    assert!(c_code.contains("int64_t ok_value;"));
    assert!(c_code.contains("MyError err_value;"));
    
    // Check for helper functions
    assert!(c_code.contains("Result_i64_MyError ok_i64_MyError(int64_t val)"));
    assert!(c_code.contains("Result_i64_MyError err_i64_MyError(MyError val)"));

    // Check for variable declarations
    assert!(c_code.contains("Result_i64_MyError x = ok_i64_MyError(10);"));
    assert!(c_code.contains("Result_i64_MyError y = err_i64_MyError(my_error_instance);"));
}
