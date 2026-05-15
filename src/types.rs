// src/types.rs
// ScrapeC Type System (Type Checking and Inference Engine)

use crate::parser::{AstNode, Pattern};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Char,
    Str,
    Unit,
    Custom(String),
    Ref(Box<Type>),
    MutRef(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Vec(Box<Type>),
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    U64,
    USize,
    F32,
    Unknown,
}

#[derive(Clone)]
pub struct TypeEnv {
    vars: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv { vars: HashMap::new() }
    }
    pub fn insert(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
    pub fn get_all_types(&self) -> impl Iterator<Item = &Type> {
        self.vars.values()
    }
}

fn check_pattern(pattern: &Pattern, value_type: &Type, env: &mut TypeEnv) {
    match (pattern, value_type) {
        (Pattern::Identifier(name), ty) => {
            env.insert(name.clone(), ty.clone());
        }
        (Pattern::Some(inner_pattern), Type::Option(inner_type)) => {
            check_pattern(inner_pattern, inner_type, env);
        }
        (Pattern::None, Type::Option(_)) => {}
        (Pattern::Ok(inner_pattern), Type::Result(ok_type, _)) => {
            check_pattern(inner_pattern, ok_type, env);
        }
        (Pattern::Err(inner_pattern), Type::Result(_, err_type)) => {
            check_pattern(inner_pattern, err_type, env);
        }
        (Pattern::Wildcard, _) => {}
        _ => panic!("Pattern {:?} does not match type {:?}", pattern, value_type),
    }
}

pub fn infer_type(ast: &AstNode, env: &mut TypeEnv) -> Type {
    match ast {
        AstNode::Let { name, type_annotation, value, mutable: _ } => {
            let mut inferred_type = infer_type(value, env);

            let final_type = if let Some(annotated_type) = type_annotation {
                if let Type::Result(ref mut inferred_ok, ref mut inferred_err) = inferred_type {
                    if let Type::Result(ref annotated_ok, ref annotated_err) = annotated_type {
                        if **inferred_ok == Type::Unknown { *inferred_ok = annotated_ok.clone(); }
                        if **inferred_err == Type::Unknown { *inferred_err = annotated_err.clone(); }
                    }
                }
                if *annotated_type != inferred_type && inferred_type != Type::Unknown {
                    panic!("Type mismatch for '{}': annotated as {:?}, but value is {:?}", name, annotated_type, inferred_type);
                }
                annotated_type.clone()
            } else {
                inferred_type
            };

            if final_type == Type::Unknown {
                panic!("Could not infer type for variable '{}'. Please add a type annotation.", name);
            }

            env.insert(name.clone(), final_type.clone());
            final_type
        },
        AstNode::Int(_) => Type::Int,
        AstNode::Float(_) => Type::Float,
        AstNode::Bool(_) => Type::Bool,
        AstNode::String(_) => Type::Str,
        AstNode::Char(_) => Type::Char,
        AstNode::Some(val) => Type::Option(Box::new(infer_type(val, env))),
        AstNode::None => Type::Unknown,
        AstNode::Ok(val) => Type::Result(Box::new(infer_type(val, env)), Box::new(Type::Unknown)),
        AstNode::Err(val) => Type::Result(Box::new(Type::Unknown), Box::new(infer_type(val, env))),
        AstNode::BinaryOp { op: _, left, right } => {
            let left_type = infer_type(left, env);
            let right_type = infer_type(right, env);
            match (&left_type, &right_type) {
                (Type::Int, Type::Int) => Type::Int,
                (Type::Float, Type::Float) => Type::Float,
                _ => panic!("Type mismatch in binary operation: {:?} and {:?}", left_type, right_type),
            }
        }
        AstNode::UnaryOp { op: _, operand } => infer_type(operand, env),
        AstNode::Match { value, arms } => {
            let value_type = infer_type(value, env);
            let mut arm_types = Vec::new();

            for arm in arms {
                let mut arm_env = env.clone();
                check_pattern(&arm.pattern, &value_type, &mut arm_env);
                let arm_type = infer_type(&arm.body, &mut arm_env);
                arm_types.push(arm_type);
            }

            let first_arm_type = arm_types.get(0).expect("Match expression must have at least one arm");
            for arm_type in &arm_types {
                if arm_type != first_arm_type {
                    panic!("Match arms have incompatible types: {:?} and {:?}", arm_type, first_arm_type);
                }
            }
            first_arm_type.clone()
        }
        AstNode::Ident(name) => env.get(name).cloned().unwrap_or(Type::Unit),
        AstNode::Function { .. } => Type::Unit,
        AstNode::Block(stmts) => {
            let mut result = Type::Unit;
            for stmt in stmts { result = infer_type(stmt, env); }
            result
        }
        AstNode::If { then_body, else_body, .. } => {
            let then_type = infer_type_block(then_body, env);
            if let Some(else_body) = else_body {
                let else_type = infer_type_block(else_body, env);
                if then_type != else_type && then_type != Type::Unknown && else_type != Type::Unknown {
                    panic!("If branches have incompatible types: {:?} and {:?}", then_type, else_type);
                }
            }
            then_type
        }
        AstNode::While { body, .. } => infer_type_block(body, env),
        AstNode::For { body, .. } => infer_type_block(body, env),
        AstNode::Loop { body } => infer_type_block(body, env),
        AstNode::Call { .. } => Type::Unit,
        AstNode::FieldAccess { .. } => Type::Unit,
        AstNode::Index { .. } => Type::Unit,
        AstNode::Tuple(_) => Type::Unit,
        AstNode::Array(_) => Type::Unit,
        AstNode::Range { .. } => Type::Unit,
        AstNode::Cast { target_type, .. } => target_type.clone(),
        AstNode::Unsafe(inner) => infer_type(inner, env),
        AstNode::Await(inner) => infer_type(inner, env),
        AstNode::Grouping(inner) => infer_type(inner, env),
        AstNode::Return(expr) => {
            if let Some(e) = expr { infer_type(e, env) } else { Type::Unit }
        }
        AstNode::Break | AstNode::Continue | AstNode::Todo => Type::Unit,
        AstNode::Struct { .. } | AstNode::Enum { .. } => Type::Unit,
    }
}

fn infer_type_block(stmts: &[AstNode], env: &mut TypeEnv) -> Type {
    let mut result = Type::Unit;
    for stmt in stmts { result = infer_type(stmt, env); }
    result
}