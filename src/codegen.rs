// src/codegen.rs
// ScrapeC Code Generation (Basic Implementation)

use crate::parser::{AstNode, MatchArm, Operator, Pattern};
use crate::types::{Type, TypeEnv};
use std::collections::HashSet;

fn type_to_c_str(ty: &Type) -> String {
    match ty {
        Type::Int => "int64_t".to_string(),
        Type::Float => "double".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Char => "char".to_string(),
        Type::Str => "char*".to_string(),
        Type::Custom(name) => name.clone(),
        Type::Option(inner) => format!("Option_{}", type_to_c_str_mangled(inner)),
        Type::Result(ok, err) => format!("Result_{}_{}", type_to_c_str_mangled(ok), type_to_c_str_mangled(err)),
        Type::Vec(inner) => format!("Vec_{}", type_to_c_str_mangled(inner)),
        Type::Ref(inner) => format!("{}*", type_to_c_str(inner)),
        Type::MutRef(inner) => format!("{}*", type_to_c_str(inner)),
        Type::Tuple(types) => {
            let parts: Vec<_> = types.iter().map(type_to_c_str).collect();
            format!("Tuple_{}", parts.join("_"))
        }
        _ => "void".to_string(),
    }
}

fn type_to_c_str_mangled(ty: &Type) -> String {
    match ty {
        Type::Int => "i64".to_string(),
        Type::Float => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Char => "char".to_string(),
        Type::Str => "str".to_string(),
        Type::Custom(name) => name.clone(),
        _ => "void".to_string(),
    }
}

fn op_to_c_str(op: &Operator) -> &'static str {
    match op {
        Operator::Add => "+",
        Operator::Subtract => "-",
        Operator::Multiply => "*",
        Operator::Divide => "/",
        Operator::Modulo => "%",
        Operator::Eq => "==",
        Operator::Ne => "!=",
        Operator::Lt => "<",
        Operator::Gt => ">",
        Operator::Le => "<=",
        Operator::Ge => ">=",
        Operator::And => "&&",
        Operator::Or => "||",
        Operator::BitAnd => "&",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::Shl => "<<",
        Operator::Shr => ">>",
        Operator::Assign => "=",
        Operator::PlusEq => "+=",
        Operator::MinusEq => "-=",
        Operator::StarEq => "*=",
        Operator::SlashEq => "/=",
    }
}

fn generate_c_expr(node: &AstNode, env: &TypeEnv, type_context: Option<&Type>) -> String {
    match node {
        AstNode::Int(val) => val.to_string(),
        AstNode::Float(val) => val.to_string(),
        AstNode::Bool(val) => val.to_string(),
        AstNode::String(s) => format!("\"{}\"", s),
        AstNode::Char(c) => format!("'{}'", c),
        AstNode::Ident(name) => name.clone(),
        AstNode::Some(val) => {
            let inner_type = if let Some(Type::Option(t)) = type_context { t } else { panic!("'Some' without Option context"); };
            let mangled = type_to_c_str_mangled(inner_type);
            format!("some_{}({})", mangled, generate_c_expr(val, env, None))
        }
        AstNode::None => {
            let inner_type = if let Some(Type::Option(t)) = type_context { t } else { panic!("'None' without Option context"); };
            let mangled = type_to_c_str_mangled(inner_type);
            format!("none_{}()", mangled)
        }
        AstNode::Ok(val) => {
            let (ok_type, err_type) = if let Some(Type::Result(o, e)) = type_context { (o, e) } else { panic!("'Ok' without Result context"); };
            let ok_m = type_to_c_str_mangled(ok_type);
            let err_m = type_to_c_str_mangled(err_type);
            format!("ok_{}_{}({})", ok_m, err_m, generate_c_expr(val, env, None))
        }
        AstNode::Err(val) => {
            let (ok_type, err_type) = if let Some(Type::Result(o, e)) = type_context { (o, e) } else { panic!("'Err' without Result context"); };
            let ok_m = type_to_c_str_mangled(ok_type);
            let err_m = type_to_c_str_mangled(err_type);
            format!("err_{}_{}({})", ok_m, err_m, generate_c_expr(val, env, None))
        }
        AstNode::BinaryOp { op, left, right } => {
            format!("({} {} {})", generate_c_expr(left, env, None), op_to_c_str(op), generate_c_expr(right, env, None))
        }
        AstNode::UnaryOp { op, operand } => {
            match op {
                Operator::Subtract => format!("(-{})", generate_c_expr(operand, env, None)),
                Operator::Ne => format!("(!{})", generate_c_expr(operand, env, None)),
                _ => format!("({}{})", op_to_c_str(op), generate_c_expr(operand, env, None)),
            }
        }
        AstNode::Match { value, arms } => {
            let mut match_code = "({".to_string();
            let value_str = generate_c_expr(value, env, None);
            let value_type = infer_type(value, &mut env.clone());

            for (i, arm) in arms.iter().enumerate() {
                let mut arm_env = env.clone();
                let (condition, var_decl) = generate_pattern_check(&arm.pattern, &value_str, &value_type, &mut arm_env);

                if i > 0 { match_code.push_str(" else "); }
                match_code.push_str(&format!("if ({}) {{\n", condition));
                if let Some(decl) = var_decl {
                    match_code.push_str(&format!("        {};\n", decl));
                }
                if let Some(guard) = &arm.guard {
                    let guard_str = generate_c_expr(guard, &arm_env, None);
                    match_code.push_str(&format!("        if ({}) {{\n", guard_str));
                    match_code.push_str(&format!("            ({});\n", generate_c_expr(&arm.body, &arm_env, None)));
                    match_code.push_str("        }");
                } else {
                    match_code.push_str(&format!("        ({});\n", generate_c_expr(&arm.body, &arm_env, None)));
                }
                match_code.push_str("    }");
            }
            match_code.push_str("})");
            match_code
        }
        AstNode::Call { callee, args } => {
            let callee_str = generate_c_expr(callee, env, None);
            let args_str: Vec<_> = args.iter().map(|a| generate_c_expr(a, env, None)).collect();
            format!("{}({})", callee_str, args_str.join(", "))
        }
        AstNode::FieldAccess { object, field } => {
            format!("({}.{})", generate_c_expr(object, env, None), field)
        }
        AstNode::Index { object, index } => {
            format!("({}[{}])", generate_c_expr(object, env, None), generate_c_expr(index, env, None))
        }
        AstNode::Range { start, end, inclusive } => {
            let start_str = generate_c_expr(start, env, None);
            let end_str = generate_c_expr(end, env, None);
            if *inclusive {
                format!("{{ .start = {}, .end = {}, .inclusive = true }}", start_str, end_str)
            } else {
                format!("{{ .start = {}, .end = {}, .inclusive = false }}", start_str, end_str)
            }
        }
        AstNode::Array(items) => {
            let items_str: Vec<_> = items.iter().map(|i| generate_c_expr(i, env, None)).collect();
            format!("{{ {} }}", items_str.join(", "))
        }
        AstNode::Tuple(items) => {
            let items_str: Vec<_> = items.iter().map(|i| generate_c_expr(i, env, None)).collect();
            format!("({})", items_str.join(", "))
        }
        AstNode::Grouping(inner) => format!("({})", generate_c_expr(inner, env, None)),
        AstNode::Block(stmts) => {
            let mut code = String::new();
            for stmt in stmts { code.push_str(&generate_c_statement(stmt, env)); }
            code
        }
        _ => panic!("Cannot generate C for expression: {:?}", node),
    }
}

fn generate_pattern_check(pattern: &Pattern, value_str: &str, value_type: &Type, env: &mut TypeEnv) -> (String, Option<String>) {
    match (pattern, value_type) {
        (Pattern::Some(inner_pattern), Type::Option(inner_type)) => {
            let condition = format!("{}.is_some", value_str);
            let (var_name, var_decl) = if let Pattern::Identifier(name) = &**inner_pattern {
                let c_type = type_to_c_str(inner_type);
                (Some(name.clone()), Some(format!("{} {} = {}.value", c_type, name, value_str)))
            } else { (None, None) };
            if let Some(name) = var_name { env.insert(name, *inner_type.clone()); }
            (condition, var_decl)
        }
        (Pattern::None, Type::Option(_)) => (format!("(!{}.is_some)", value_str), None),
        (Pattern::Ok(inner_pattern), Type::Result(ok_type, _)) => {
            let condition = format!("{}.is_ok", value_str);
            let (var_name, var_decl) = if let Pattern::Identifier(name) = &**inner_pattern {
                let c_type = type_to_c_str(ok_type);
                (Some(name.clone()), Some(format!("{} {} = {}.value.ok_value", c_type, name, value_str)))
            } else { (None, None) };
            if let Some(name) = var_name { env.insert(name, *ok_type.clone()); }
            (condition, var_decl)
        }
        (Pattern::Err(inner_pattern), Type::Result(_, err_type)) => {
            let condition = format!("(!{}.is_ok)", value_str);
            let (var_name, var_decl) = if let Pattern::Identifier(name) = &**inner_pattern {
                let c_type = type_to_c_str(err_type);
                (Some(name.clone()), Some(format!("{} {} = {}.value.err_value", c_type, name, value_str)))
            } else { (None, None) };
            if let Some(name) = var_name { env.insert(name, *err_type.clone()); }
            (condition, var_decl)
        }
        (Pattern::Identifier(name), _) => {
            env.insert(name.clone(), value_type.clone());
            ("1".to_string(), Some(format!("auto {} = {}", name, value_str)))
        }
        (Pattern::Wildcard, _) => ("1".to_string(), None),
        _ => ("1".to_string(), None),
    }
}

fn generate_c_statement(node: &AstNode, env: &TypeEnv) -> String {
    match node {
        AstNode::Let { name, type_annotation, value, mutable: _ } => {
            let var_type = env.get(name).expect("Variable should have a type");
            let c_type_str = type_to_c_str(var_type);
            let expr_str = generate_c_expr(value, env, Some(var_type));
            format!("    {} {} = {};\n", c_type_str, name, expr_str)
        }
        AstNode::Return(Some(expr)) => format!("    return {};\n", generate_c_expr(expr, env, None)),
        AstNode::Return(None) => "    return;\n".to_string(),
        AstNode::If { condition, then_body, else_body } => {
            let cond_str = generate_c_expr(condition, env, None);
            let mut code = format!("    if ({}) {{\n", cond_str);
            for stmt in then_body { code.push_str(&generate_c_statement(stmt, env)); }
            code.push_str("    }");
            if let Some(else_stmts) = else_body {
                code.push_str(" else {\n");
                for stmt in else_stmts { code.push_str(&generate_c_statement(stmt, env)); }
                code.push_str("    }");
            }
            code.push('\n');
            code
        }
        AstNode::While { condition, body } => {
            let cond_str = generate_c_expr(condition, env, None);
            let mut code = format!("    while ({}) {{\n", cond_str);
            for stmt in body { code.push_str(&generate_c_statement(stmt, env)); }
            code.push_str("    }\n");
            code
        }
        AstNode::For { name, iterator, body } => {
            let iter_str = generate_c_expr(iterator, env, None);
            let mut code = format!("    for (auto {} : {}) {{\n", name, iter_str);
            for stmt in body { code.push_str(&generate_c_statement(stmt, env)); }
            code.push_str("    }\n");
            code
        }
        AstNode::Loop { body } => {
            let mut code = "    while (true) {\n".to_string();
            for stmt in body { code.push_str(&generate_c_statement(stmt, env)); }
            code.push_str("    }\n");
            code
        }
        AstNode::Break => "    break;\n".to_string(),
        AstNode::Continue => "    continue;\n".to_string(),
        AstNode::Block(stmts) => {
            let mut code = "    {\n".to_string();
            for stmt in stmts { code.push_str(&generate_c_statement(stmt, env)); }
            code.push_str("    }\n");
            code
        }
        _ => format!("    /* unimplemented: {:?} */\n", node),
    }
}

use crate::types::infer_type;

pub fn generate_c(ast: &[AstNode], env: &TypeEnv) -> String {
    let mut code = String::from("// Generated C code\n#include <stdio.h>\n#include <stdbool.h>\n#include <stdint.h>\n\n");
    let mut defined_structs = HashSet::new();

    for ty in env.get_all_types() {
        if let Type::Custom(name) = ty {
            if !defined_structs.contains(name) {
                code.push_str(&format!("typedef struct {} {{ int dummy; }} {};\n\n", name, name));
                defined_structs.insert(name.clone());
            }
        }
        match ty {
            Type::Option(inner) => {
                let struct_name = type_to_c_str(ty);
                if !defined_structs.contains(&struct_name) {
                    let c_type_str = type_to_c_str(inner);
                    let c_type_mangled = type_to_c_str_mangled(inner);
                    code.push_str(&format!("typedef struct {} {{ bool is_some; {} value; }} {};\n", struct_name, c_type_str, struct_name));
                    code.push_str(&format!("{} some_{}({} val) {{ {} opt = {{.is_some = true, .value = val}}; return opt; }}\n", struct_name, c_type_mangled, c_type_str, struct_name));
                    code.push_str(&format!("{} none_{}() {{ {} opt = {{.is_some = false}}; return opt; }}\n\n", struct_name, c_type_mangled, struct_name));
                    defined_structs.insert(struct_name);
                }
            }
            Type::Result(ok_type, err_type) => {
                let struct_name = type_to_c_str(ty);
                if !defined_structs.contains(&struct_name) {
                    let ok_c_type = type_to_c_str(ok_type);
                    let err_c_type = type_to_c_str(err_type);
                    let ok_mangled = type_to_c_str_mangled(ok_type);
                    let err_mangled = type_to_c_str_mangled(err_type);
                    code.push_str(&format!("typedef struct {} {{ bool is_ok; union {{ {} ok_value; {} err_value; }} value; }} {};\n", struct_name, ok_c_type, err_c_type, struct_name));
                    code.push_str(&format!("{} ok_{}_{}({} val) {{ {} res = {{.is_ok = true}}; res.value.ok_value = val; return res; }}\n", struct_name, ok_mangled, err_mangled, ok_c_type, struct_name));
                    code.push_str(&format!("{} err_{}_{}({} val) {{ {} res = {{.is_ok = false}}; res.value.err_value = val; return res; }}\n\n", struct_name, ok_mangled, err_mangled, err_c_type, struct_name));
                    defined_structs.insert(struct_name);
                }
            }
            _ => {}
        }
    }

    for node in ast {
        match node {
            AstNode::Function { name, params, return_type, body, .. } => {
                let ret_type_str = return_type.as_ref().map_or("void".to_string(), |t| type_to_c_str(t));
                let params_str: Vec<_> = params.iter().map(|p| {
                    format!("{} {}", p.ty.as_ref().map_or("void*".to_string(), |t| type_to_c_str(t)), p.name)
                }).collect();
                code.push_str(&format!("{} {}({}) {{\n", ret_type_str, name, params_str.join(", ")));
                for stmt in body { code.push_str(&generate_c_statement(stmt, env)); }
                code.push_str("}\n\n");
            }
            AstNode::Struct { name, fields } => {
                code.push_str(&format!("typedef struct {} {{\n", name));
                for field in fields {
                    code.push_str(&format!("    {} {};\n", type_to_c_str(&field.ty), field.name));
                }
                code.push_str(&format!("}} {};\n\n", name));
            }
            AstNode::Enum { name, variants } => {
                code.push_str(&format!("typedef enum {} {{\n", name));
                for variant in variants {
                    match variant {
                        crate::parser::EnumVariant::Unit(v) => code.push_str(&format!("    {},\n", v)),
                        crate::parser::EnumVariant::Tuple(v, _) => code.push_str(&format!("    {},\n", v)),
                        crate::parser::EnumVariant::Struct(v, _) => code.push_str(&format!("    {},\n", v)),
                    }
                }
                code.push_str(&format!("}} {};\n\n", name));
            }
            AstNode::Let { name, value, .. } => {
                if let Some(var_type) = env.get(name) {
                    let c_type_str = type_to_c_str(var_type);
                    let expr_str = generate_c_expr(value, env, Some(var_type));
                    code.push_str(&format!("    {} {} = {};\n", c_type_str, name, expr_str));
                }
            }
            _ => {}
        }
    }
    code
}