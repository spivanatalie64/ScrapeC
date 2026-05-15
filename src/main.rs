// src/main.rs
// ScrapeC Reference Compiler (Stub)

use scrapec::lexer::{lex, Token};
use scrapec::parser::{parse, AstNode};
use scrapec::types::{self, TypeEnv};
use scrapec::{codegen, ffi, macros, asyncmod, smartptr, r#unsafe as sc_unsafe, doc};

fn main() {
    let source = "fn main() { let x = 42; }";
    let tokens = lex(source);
    println!("ScrapeC tokens: {:?}", tokens);
    let ast = parse(&tokens);
    println!("ScrapeC AST: {:?}", ast);
    // Type inference
    let mut type_env = TypeEnv::new();
    for node in &ast {
        let ty = types::infer_type(node, &mut type_env);
        println!("Type: {:?}", ty);
    }
    // Codegen (stub)
    let c_code = codegen::generate_c(&ast, &type_env);
    println!("Generated C code:\n{}", c_code);
    // FFI header (stub)
    println!("C header:\n{}", ffi::generate_c_header());
    println!("Rust extern mod:\n{}", ffi::generate_rust_mod());
    // Macro expansion (stub)
    let expanded = macros::expand_macros(source);
    println!("Macro-expanded source:\n{}", expanded);
    // Async/concurrency (stub)
    asyncmod::spawn_async(|| println!("Async task!"));
    // Smart pointer (stub)
    let _boxed = smartptr::Boxed::new(123);
    // Unsafe (stub)
    unsafe {
        let _ptr = sc_unsafe::raw_ptr(&_boxed);
    }
    // Doc/test (stub)
    let _docs = doc::extract_docs(source);
    doc::run_tests();
}
