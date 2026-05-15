// src/parser.rs
// ScrapeC Parser: Parses tokens into an AST

use crate::lexer::Token;
use crate::types::Type;
use std::slice::Iter;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Assign,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Identifier(String),
    Some(Box<Pattern>),
    None,
    Ok(Box<Pattern>),
    Err(Box<Pattern>),
    Wildcard,
    Tuple(Vec<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<AstNode>>,
    pub body: Box<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub mutable: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnumVariant {
    Unit(String),
    Tuple(String, Vec<Type>),
    Struct(String, Vec<StructField>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Function {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<AstNode>,
        is_async: bool,
        is_unsafe: bool,
        is_pub: bool,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    Let {
        name: String,
        type_annotation: Option<Type>,
        value: Box<AstNode>,
        mutable: bool,
    },
    Return(Option<Box<AstNode>>),
    If {
        condition: Box<AstNode>,
        then_body: Vec<AstNode>,
        else_body: Option<Vec<AstNode>>,
    },
    While {
        condition: Box<AstNode>,
        body: Vec<AstNode>,
    },
    For {
        name: String,
        iterator: Box<AstNode>,
        body: Vec<AstNode>,
    },
    Loop {
        body: Vec<AstNode>,
    },
    Break,
    Continue,
    Block(Vec<AstNode>),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Ident(String),
    Some(Box<AstNode>),
    None,
    Ok(Box<AstNode>),
    Err(Box<AstNode>),
    BinaryOp {
        op: Operator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UnaryOp {
        op: Operator,
        operand: Box<AstNode>,
    },
    Match {
        value: Box<AstNode>,
        arms: Vec<MatchArm>,
    },
    Call {
        callee: Box<AstNode>,
        args: Vec<AstNode>,
    },
    FieldAccess {
        object: Box<AstNode>,
        field: String,
    },
    Index {
        object: Box<AstNode>,
        index: Box<AstNode>,
    },
    Tuple(Vec<AstNode>),
    Array(Vec<AstNode>),
    Range {
        start: Box<AstNode>,
        end: Box<AstNode>,
        inclusive: bool,
    },
    Cast {
        expr: Box<AstNode>,
        target_type: Type,
    },
    Unsafe(Box<AstNode>),
    Await(Box<AstNode>),
    Grouping(Box<AstNode>),
    Todo,
}

struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens: tokens.iter().peekable() }
    }

    fn parse(&mut self) -> Vec<AstNode> {
        let mut ast = Vec::new();
        while self.tokens.peek().is_some() && **self.tokens.peek().unwrap() != Token::Eof {
            ast.push(self.parse_statement());
        }
        ast
    }

    fn parse_statement(&mut self) -> AstNode {
        match self.tokens.peek() {
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::For) => self.parse_for_statement(),
            Some(Token::Loop) => self.parse_loop_statement(),
            Some(Token::Break) => { self.tokens.next(); AstNode::Break },
            Some(Token::Continue) => { self.tokens.next(); AstNode::Continue },
            Some(Token::Struct) => self.parse_struct_definition(),
            Some(Token::Enum) => self.parse_enum_definition(),
            Some(Token::Fn) => self.parse_function_definition(),
            Some(Token::Unsafe) => {
                self.tokens.next();
                match self.tokens.peek() {
                    Some(Token::LBrace) => AstNode::Unsafe(Box::new(AstNode::Block(self.parse_block()))),
                    _ => AstNode::Unsafe(Box::new(self.parse_expression(0))),
                }
            }
            Some(_) => {
                let expr = self.parse_expression(0);
                if self.tokens.peek() == Some(&&Token::Semicolon) { self.tokens.next(); }
                expr
            }
            None => panic!("Unexpected EOF"),
        }
    }

    fn parse_block(&mut self) -> Vec<AstNode> {
        assert_eq!(self.tokens.next(), Some(&Token::LBrace));
        let mut body = Vec::new();
        while self.tokens.peek() != Some(&&Token::RBrace) {
            body.push(self.parse_statement());
        }
        assert_eq!(self.tokens.next(), Some(&Token::RBrace));
        body
    }

    fn parse_function_definition(&mut self) -> AstNode {
        self.tokens.next();
        let is_async = self.tokens.peek() == Some(&&Token::Async) && { self.tokens.next(); true };
        let is_unsafe = self.tokens.peek() == Some(&&Token::Unsafe) && { self.tokens.next(); true };
        let is_pub = self.tokens.peek() == Some(&&Token::Pub) && { self.tokens.next(); true };

        let name = match self.tokens.next() {
            Some(Token::Ident(n)) => n.clone(),
            _ => panic!("Expected function name"),
        };
        assert_eq!(self.tokens.next(), Some(&Token::LParen));

        let mut params = Vec::new();
        while self.tokens.peek() != Some(&&Token::RParen) {
            let mut param_mut = false;
            if self.tokens.peek() == Some(&&Token::Mut) { self.tokens.next(); param_mut = true; }
            let param_name = match self.tokens.next() {
                Some(Token::Ident(n)) => n.clone(),
                _ => panic!("Expected parameter name"),
            };
            let param_type = if self.tokens.peek() == Some(&&Token::Colon) {
                self.tokens.next();
                Some(self.parse_type())
            } else { None };
            params.push(Param { name: param_name, ty: param_type, mutable: param_mut });
            if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); }
        }
        assert_eq!(self.tokens.next(), Some(&Token::RParen));

        let return_type = if self.tokens.peek() == Some(&&Token::ThinArrow) {
            self.tokens.next();
            Some(self.parse_type())
        } else { None };

        let body = self.parse_block();
        AstNode::Function { name, params, return_type, body, is_async, is_unsafe, is_pub }
    }

    fn parse_let_statement(&mut self) -> AstNode {
        self.tokens.next();
        let mutable = self.tokens.peek() == Some(&&Token::Mut) && { self.tokens.next(); true };
        let name = match self.tokens.next() {
            Some(Token::Ident(n)) => n.clone(),
            _ => panic!("Expected identifier after 'let'"),
        };
        let type_annotation = if self.tokens.peek() == Some(&&Token::Colon) {
            self.tokens.next();
            Some(self.parse_type())
        } else { None };
        assert_eq!(self.tokens.next(), Some(&Token::Assign));
        let value = self.parse_expression(0);
        if self.tokens.peek() == Some(&&Token::Semicolon) { self.tokens.next(); }
        AstNode::Let { name, type_annotation, value: Box::new(value), mutable }
    }

    fn parse_return_statement(&mut self) -> AstNode {
        self.tokens.next();
        let expr = if self.tokens.peek() != Some(&&Token::Semicolon) && self.tokens.peek() != Some(&&Token::RBrace) {
            Some(Box::new(self.parse_expression(0)))
        } else { None };
        if self.tokens.peek() == Some(&&Token::Semicolon) { self.tokens.next(); }
        AstNode::Return(expr)
    }

    fn parse_if_statement(&mut self) -> AstNode {
        self.tokens.next();
        assert_eq!(self.tokens.next(), Some(&Token::LParen));
        let condition = self.parse_expression(0);
        assert_eq!(self.tokens.next(), Some(&Token::RParen));
        let then_body = self.parse_block();
        let else_body = if self.tokens.peek() == Some(&&Token::Else) {
            self.tokens.next();
            if self.tokens.peek() == Some(&&Token::If) {
                let inner = self.parse_if_statement();
                match inner {
                    AstNode::If { then_body, else_body, .. } => {
                        let mut combined = vec![AstNode::Block(then_body)];
                        combined.extend(else_body.unwrap_or_default());
                        Some(combined)
                    }
                    _ => Some(vec![inner]),
                }
            } else { Some(self.parse_block()) }
        } else { None };
        AstNode::If { condition: Box::new(condition), then_body, else_body }
    }

    fn parse_while_statement(&mut self) -> AstNode {
        self.tokens.next();
        assert_eq!(self.tokens.next(), Some(&Token::LParen));
        let condition = self.parse_expression(0);
        assert_eq!(self.tokens.next(), Some(&Token::RParen));
        AstNode::While { condition: Box::new(condition), body: self.parse_block() }
    }

    fn parse_for_statement(&mut self) -> AstNode {
        self.tokens.next();
        let name = match self.tokens.next() {
            Some(Token::Ident(n)) => n.clone(),
            _ => panic!("Expected iterator variable name"),
        };
        assert_eq!(self.tokens.next(), Some(&Token::In));
        let iterator = self.parse_expression(0);
        AstNode::For { name, iterator: Box::new(iterator), body: self.parse_block() }
    }

    fn parse_loop_statement(&mut self) -> AstNode {
        self.tokens.next();
        AstNode::Loop { body: self.parse_block() }
    }

    fn parse_struct_definition(&mut self) -> AstNode {
        self.tokens.next(); // struct
        let name = match self.tokens.next() {
            Some(Token::Ident(n)) => n.clone(),
            _ => panic!("Expected struct name"),
        };
        assert_eq!(self.tokens.next(), Some(&Token::LBrace)); // consume '{'
        let mut fields = Vec::new();
        while self.tokens.peek() != Some(&&Token::RBrace) {
            let _is_pub = self.tokens.peek() == Some(&&Token::Pub) && { self.tokens.next(); true };
            let field_name = match self.tokens.next() {
                Some(Token::Ident(n)) => n.clone(),
                _ => panic!("Expected field name"),
            };
            assert_eq!(self.tokens.next(), Some(&Token::Colon));
            fields.push(StructField { name: field_name, ty: self.parse_type() });
            if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); }
        }
        assert_eq!(self.tokens.next(), Some(&Token::RBrace));
        AstNode::Struct { name, fields }
    }

    fn parse_enum_definition(&mut self) -> AstNode {
        self.tokens.next(); // enum
        let name = match self.tokens.next() {
            Some(Token::Ident(n)) => n.clone(),
            _ => panic!("Expected enum name"),
        };
        assert_eq!(self.tokens.next(), Some(&Token::LBrace)); // consume '{'
        let mut variants = Vec::new();
        while self.tokens.peek() != Some(&&Token::RBrace) {
            let variant_name = match self.tokens.next() {
                Some(Token::Ident(n)) => n.clone(),
                _ => panic!("Expected variant name"),
            };
            let variant = if self.tokens.peek() == Some(&&Token::LParen) {
                self.tokens.next();
                let mut types = Vec::new();
                while self.tokens.peek() != Some(&&Token::RParen) { types.push(self.parse_type()); if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); } }
                self.tokens.next();
                EnumVariant::Tuple(variant_name, types)
            } else if self.tokens.peek() == Some(&&Token::LBrace) {
                self.tokens.next();
                let mut fields = Vec::new();
                while self.tokens.peek() != Some(&&Token::RBrace) {
                    let field_name = match self.tokens.next() {
                        Some(Token::Ident(n)) => n.clone(),
                        _ => panic!("Expected field name"),
                    };
                    assert_eq!(self.tokens.next(), Some(&Token::Colon));
                    fields.push(StructField { name: field_name, ty: self.parse_type() });
                    if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); }
                }
                self.tokens.next();
                EnumVariant::Struct(variant_name, fields)
            } else {
                EnumVariant::Unit(variant_name)
            };
            variants.push(variant);
            if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); }
        }
        assert_eq!(self.tokens.next(), Some(&Token::RBrace));
        AstNode::Enum { name, variants }
    }

    fn parse_type(&mut self) -> Type {
        let want_mut_ref = self.tokens.peek() == Some(&&Token::Mut);
        if want_mut_ref { self.tokens.next(); }
        let want_ref = self.tokens.peek() == Some(&&Token::Ref);
        if want_ref { self.tokens.next(); }

        let ty = match self.tokens.next() {
            Some(Token::Ident(name)) => match name.as_str() {
                "int" => Type::Int, "float" => Type::Float, "bool" => Type::Bool,
                "char" => Type::Char, "str" => Type::Str,
                "i8" => Type::I8, "i16" => Type::I16, "i32" => Type::I32, "i64" => Type::Int,
                "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32,
                "u64" => Type::U64, "usize" => Type::USize,
                "f32" => Type::F32, "f64" => Type::Float,
                "Option" => {
                    assert_eq!(self.tokens.next(), Some(&Token::LessThan));
                    let inner = self.parse_type();
                    assert_eq!(self.tokens.next(), Some(&Token::GreaterThan));
                    Type::Option(Box::new(inner))
                }
                "Result" => {
                    assert_eq!(self.tokens.next(), Some(&Token::LessThan));
                    let ok = self.parse_type();
                    assert_eq!(self.tokens.next(), Some(&Token::Comma));
                    let err = self.parse_type();
                    assert_eq!(self.tokens.next(), Some(&Token::GreaterThan));
                    Type::Result(Box::new(ok), Box::new(err))
                }
                "Vec" => {
                    assert_eq!(self.tokens.next(), Some(&Token::LessThan));
                    let inner = self.parse_type();
                    assert_eq!(self.tokens.next(), Some(&Token::GreaterThan));
                    Type::Vec(Box::new(inner))
                }
                _ => Type::Custom(name.clone()),
            },
            Some(Token::LParen) => {
                let mut types = Vec::new();
                if self.tokens.peek() != Some(&&Token::RParen) {
                    types.push(self.parse_type());
                    while self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); types.push(self.parse_type()); }
                }
                assert_eq!(self.tokens.next(), Some(&Token::RParen));
                if types.len() == 1 { types.into_iter().next().unwrap() } else { Type::Tuple(types) }
            }
            _ => panic!("Expected type name"),
        };

        if want_ref { Type::Ref(Box::new(ty)) } else if want_mut_ref { Type::MutRef(Box::new(ty)) } else { ty }
    }

    fn parse_expression(&mut self, precedence: u8) -> AstNode {
        let mut left = self.parse_prefix();
        while let Some(op) = self.tokens.peek() {
            let op_precedence = Self::get_precedence(op);
            if precedence >= op_precedence { break; }
            left = self.parse_infix(left);
        }
        left
    }

    fn parse_prefix(&mut self) -> AstNode {
        match self.tokens.next() {
            Some(Token::Match) | Some(Token::Ident(ref m)) if m == "match" => self.parse_match_expression(),
            Some(Token::If) => {
                assert_eq!(self.tokens.next(), Some(&Token::LParen));
                let condition = self.parse_expression(0);
                assert_eq!(self.tokens.next(), Some(&Token::RParen));
                let then_body = self.parse_block();
                let else_body = if self.tokens.peek() == Some(&&Token::Else) {
                    self.tokens.next();
                    Some(self.parse_block())
                } else { None };
                AstNode::If { condition: Box::new(condition), then_body, else_body }
            }
            Some(Token::Async) => AstNode::Await(Box::new(self.parse_expression(0))),
            Some(Token::Await) => AstNode::Await(Box::new(self.parse_expression(0))),
            Some(Token::Not) => {
                let _operand = self.parse_expression(10);
                AstNode::UnaryOp { op: Operator::Ne, operand: Box::new(AstNode::Bool(true)) }
            }
            Some(Token::Minus) => {
                let operand = self.parse_expression(10);
                AstNode::UnaryOp { op: Operator::Subtract, operand: Box::new(operand) }
            }
            Some(Token::Int(val)) => AstNode::Int(*val),
            Some(Token::Float(val)) => AstNode::Float(*val),
            Some(Token::Bool(val)) => AstNode::Bool(*val),
            Some(Token::String(ref s)) => AstNode::String(s.clone()),
            Some(Token::Char(c)) => AstNode::Char(*c),
            Some(Token::Ident(name)) => AstNode::Ident(name.clone()),
            Some(Token::Some) => self.parse_constructor(AstNode::Some),
            Some(Token::None) => AstNode::None,
            Some(Token::Ok) => self.parse_constructor(AstNode::Ok),
            Some(Token::Err) => self.parse_constructor(AstNode::Err),
            Some(Token::LParen) => {
                let expr = self.parse_expression(0);
                if self.tokens.peek() == Some(&&Token::RParen) {
                    self.tokens.next();
                    AstNode::Grouping(Box::new(expr))
                } else {
                    let mut exprs = vec![expr];
                    while self.tokens.peek() != Some(&&Token::RParen) {
                        assert_eq!(self.tokens.next(), Some(&Token::Comma));
                        exprs.push(self.parse_expression(0));
                    }
                    self.tokens.next();
                    if exprs.len() == 1 { AstNode::Grouping(Box::new(exprs.into_iter().next().unwrap())) }
                    else { AstNode::Tuple(exprs) }
                }
            }
            Some(Token::LBracket) => {
                let mut exprs = Vec::new();
                while self.tokens.peek() != Some(&&Token::RBracket) {
                    exprs.push(self.parse_expression(0));
                    if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); }
                }
                assert_eq!(self.tokens.next(), Some(&Token::RBracket));
                AstNode::Array(exprs)
            }
            t => panic!("Unexpected token in prefix position: {:?}", t),
        }
    }

    fn parse_constructor(&mut self, constructor: fn(Box<AstNode>) -> AstNode) -> AstNode {
        assert_eq!(self.tokens.next(), Some(&Token::LParen));
        let expr = self.parse_expression(0);
        assert_eq!(self.tokens.next(), Some(&Token::RParen));
        constructor(Box::new(expr))
    }

    fn parse_match_expression(&mut self) -> AstNode {
        let value = self.parse_expression(0);
        assert_eq!(self.tokens.next(), Some(&Token::LBrace));
        let mut arms = Vec::new();
        while self.tokens.peek() != Some(&&Token::RBrace) {
            let pattern = self.parse_pattern();
            let guard = if let Some(&Token::Ident(ref s)) = self.tokens.peek() {
                if s == "if" {
                    self.tokens.next();
                    Some(Box::new(self.parse_expression(0)))
                } else { None }
            } else { None };
            assert_eq!(self.tokens.next(), Some(&Token::FatArrow));
            let body = self.parse_expression(0);
            arms.push(MatchArm { pattern, guard, body: Box::new(body) });
            if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); }
        }
        assert_eq!(self.tokens.next(), Some(&Token::RBrace));
        AstNode::Match { value: Box::new(value), arms }
    }

    fn parse_pattern(&mut self) -> Pattern {
        match self.tokens.next() {
            Some(Token::Some) => {
                assert_eq!(self.tokens.next(), Some(&Token::LParen));
                let inner = self.parse_pattern();
                assert_eq!(self.tokens.next(), Some(&Token::RParen));
                Pattern::Some(Box::new(inner))
            }
            Some(Token::None) => Pattern::None,
            Some(Token::Ok) => {
                assert_eq!(self.tokens.next(), Some(&Token::LParen));
                let inner = self.parse_pattern();
                assert_eq!(self.tokens.next(), Some(&Token::RParen));
                Pattern::Ok(Box::new(inner))
            }
            Some(Token::Err) => {
                assert_eq!(self.tokens.next(), Some(&Token::LParen));
                let inner = self.parse_pattern();
                assert_eq!(self.tokens.next(), Some(&Token::RParen));
                Pattern::Err(Box::new(inner))
            }
            Some(Token::Ident(name)) if name == "_" => Pattern::Wildcard,
            Some(Token::Ident(name)) => Pattern::Identifier(name.clone()),
            t => panic!("Unsupported pattern: {:?}", t),
        }
    }

    fn parse_infix(&mut self, left: AstNode) -> AstNode {
        let op_token = self.tokens.next().unwrap();
        let precedence = Self::get_precedence(op_token);

        // Handle range operators (dotdot / range-inclusive)
        if matches!(op_token, Token::DotDot | Token::RangeInclusive) {
            let inclusive = matches!(op_token, Token::RangeInclusive);
            let right = self.parse_expression(0);
            return AstNode::Range { start: Box::new(left), end: Box::new(right), inclusive };
        }

        // Handle dot (field access)
        if matches!(op_token, Token::Dot) {
            if self.tokens.peek() == Some(&&Token::Dot) {
                self.tokens.next();
                let inclusive = self.tokens.peek() == Some(&&Token::Assign);
                if inclusive { self.tokens.next(); }
                let right = self.parse_expression(0);
                return AstNode::Range { start: Box::new(left), end: Box::new(right), inclusive };
            }
            let field = match self.tokens.next() {
                Some(Token::Ident(name)) => name.clone(),
                _ => panic!("Expected field name after '.'"),
            };
            return AstNode::FieldAccess { object: Box::new(left), field };
        }

        if matches!(op_token, Token::LParen) {
            let mut args = Vec::new();
            while self.tokens.peek() != Some(&&Token::RParen) { args.push(self.parse_expression(0)); if self.tokens.peek() == Some(&&Token::Comma) { self.tokens.next(); } }
            assert_eq!(self.tokens.next(), Some(&Token::RParen));
            return AstNode::Call { callee: Box::new(left), args };
        }

        if matches!(op_token, Token::LBracket) {
            let index = self.parse_expression(0);
            assert_eq!(self.tokens.next(), Some(&Token::RBracket));
            return AstNode::Index { object: Box::new(left), index: Box::new(index) };
        }

        let op = match op_token {
            Token::Plus => Operator::Add, Token::Minus => Operator::Subtract,
            Token::Star => Operator::Multiply, Token::Slash => Operator::Divide,
            Token::Percent => Operator::Modulo, Token::Eq => Operator::Eq,
            Token::Ne => Operator::Ne, Token::LessThan => Operator::Lt,
            Token::GreaterThan => Operator::Gt, Token::Le => Operator::Le,
            Token::Ge => Operator::Ge, Token::AndAnd => Operator::And,
            Token::OrOr => Operator::Or, Token::BitAnd => Operator::BitAnd,
            Token::BitOr => Operator::BitOr, Token::BitXor => Operator::BitXor,
            Token::Shl => Operator::Shl, Token::Shr => Operator::Shr,
            Token::Assign => Operator::Assign, Token::PlusEq => Operator::PlusEq,
            Token::MinusEq => Operator::MinusEq, Token::StarEq => Operator::StarEq,
            Token::SlashEq => Operator::SlashEq,
            _ => panic!("Invalid infix token: {:?}", op_token),
        };
        let right = self.parse_expression(precedence);
        AstNode::BinaryOp { op, left: Box::new(left), right: Box::new(right) }
    }

    fn get_precedence(token: &Token) -> u8 {
        match token {
            Token::DotDot | Token::RangeInclusive => 15,
            Token::Dot | Token::DoubleColon => 14,
            Token::LBracket | Token::LParen => 13,
            Token::As => 12,
            Token::Star | Token::Slash | Token::Percent => 11,
            Token::Plus | Token::Minus => 10,
            Token::Shl | Token::Shr => 9,
            Token::BitAnd => 8, Token::BitXor => 7, Token::BitOr => 6,
            Token::Eq | Token::Ne | Token::LessThan | Token::GreaterThan | Token::Le | Token::Ge => 5,
            Token::AndAnd => 4, Token::OrOr => 3,
            Token::Assign | Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq => 1,
            _ => 0,
        }
    }
}

pub fn parse(tokens: &[Token]) -> Vec<AstNode> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token> { super::super::lexer::lex(input) }

    #[test]
    fn test_parse_function() {
        let ast = parse(&lex("fn foo() { let x = 5; }"));
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Function { name, params, .. } => { assert_eq!(name, "foo"); assert!(params.is_empty()); }
            _ => panic!("Expected Function node"),
        }
    }

    #[test]
    fn test_parse_function_with_params() {
        match &parse(&lex("fn add(a: int, b: int) -> int { a + b }"))[0] {
            AstNode::Function { name, params, return_type, .. } => {
                assert_eq!(name, "add"); assert_eq!(params.len(), 2);
                assert!(matches!(return_type, Some(Type::Int)));
            }
            _ => panic!("Expected Function node"),
        }
    }

    #[test]
    fn test_if_else() {
        match &parse(&lex("if (x > 0) { 1 } else { 0 }"))[0] {
            AstNode::If { then_body, else_body, .. } => { assert!(!then_body.is_empty()); assert!(else_body.is_some()); }
            _ => panic!("Expected If node"),
        }
    }

    #[test]
    fn test_struct_definition() {
        match &parse(&lex("struct Point { x: int, y: int }"))[0] {
            AstNode::Struct { name, fields } => { assert_eq!(name, "Point"); assert_eq!(fields.len(), 2); }
            _ => panic!("Expected Struct node"),
        }
    }

    #[test]
    fn test_enum_definition() {
        match &parse(&lex("enum Color { Red, Green(rgb: int), Blue { shade: int } }"))[0] {
            AstNode::Enum { name, variants } => { assert_eq!(name, "Color"); assert_eq!(variants.len(), 3); }
            _ => panic!("Expected Enum node"),
        }
    }

    #[test]
    fn test_for_loop() {
        match &parse(&lex("for i in 0..10 { print(i); }"))[0] {
            AstNode::For { name, .. } => assert_eq!(name, "i"),
            _ => panic!("Expected For node"),
        }
    }

    #[test]
    fn test_while_loop() {
        assert!(matches!(&parse(&lex("while (x > 0) { x = x - 1; }"))[0], AstNode::While { .. }));
    }

    #[test]
    fn test_return() {
        match &parse(&lex("fn foo() -> int { return 42; }"))[0] {
            AstNode::Function { body, .. } => {
                match &body[0] {
                    AstNode::Return(Some(val)) => assert!(matches!(&**val, AstNode::Int(42))),
                    _ => panic!("Expected Return node"),
                }
            }
            _ => panic!("Expected Function node"),
        }
    }

    #[test]
    fn test_match_expression() {
        assert!(matches!(&parse(&lex("match x { Some(v) => v, None => 0 }"))[0],
            AstNode::Match { arms, .. } if arms.len() == 2));
    }

    #[test]
    fn test_array_literal() {
        match &parse(&lex("[1, 2, 3]"))[0] {
            AstNode::Array(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected Array node"),
        }
    }

    #[test]
    fn test_call_expression() {
        match &parse(&lex("foo(1, 2)"))[0] {
            AstNode::Call { callee, args } => {
                assert!(matches!(&**callee, AstNode::Ident(s) if s == "foo"));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call node"),
        }
    }

    #[test]
    fn test_field_access() {
        match &parse(&lex("point.x"))[0] {
            AstNode::FieldAccess { object, field } => {
                assert!(matches!(&**object, AstNode::Ident(s) if s == "point"));
                assert_eq!(field, "x");
            }
            _ => panic!("Expected FieldAccess node"),
        }
    }

    #[test]
    fn test_index_access() {
        match &parse(&lex("arr[0]"))[0] {
            AstNode::Index { object, index } => {
                assert!(matches!(&**object, AstNode::Ident(s) if s == "arr"));
                assert!(matches!(&**index, AstNode::Int(0)));
            }
            _ => panic!("Expected Index node"),
        }
    }

    #[test]
    fn test_string_literal() {
        match &parse(&lex(r#"let x = "hello";"#))[0] {
            AstNode::Let { value, .. } => assert!(matches!(&**value, AstNode::String(s) if s == "hello")),
            _ => panic!("Expected Let with String"),
        }
    }

    #[test]
    fn test_char_literal() {
        match &parse(&lex("let x = 'a';"))[0] {
            AstNode::Let { value, .. } => assert!(matches!(&**value, AstNode::Char('a'))),
            _ => panic!("Expected Let with Char"),
        }
    }

    #[test]
    fn test_range_expression() {
        match &parse(&lex("0..10"))[0] {
            AstNode::Range { start, end, inclusive } => {
                assert!(matches!(&**start, AstNode::Int(0)));
                assert!(matches!(&**end, AstNode::Int(10)));
                assert!(!inclusive);
            }
            _ => panic!("Expected Range node"),
        }
    }

    #[test]
    fn test_range_inclusive_expression() {
        assert!(matches!(&parse(&lex("0..=10"))[0], AstNode::Range { inclusive: true, .. }));
    }

    #[test]
    fn test_mut_param() {
        match &parse(&lex("fn foo(mut x: int) { }"))[0] {
            AstNode::Function { params, .. } => assert!(params[0].mutable),
            _ => panic!("Expected Function node"),
        }
    }

    #[test]
    fn test_if_else_if_else() {
        match &parse(&lex("if (x > 0) { 1 } else if (x < 0) { -1 } else { 0 }"))[0] {
            AstNode::If { then_body, else_body, .. } => { assert!(!then_body.is_empty()); assert!(else_body.is_some()); }
            _ => panic!("Expected If node"),
        }
    }

    #[test]
    fn test_ref_type() {
        match &parse(&lex("fn foo(x: ref int) { }"))[0] {
            AstNode::Function { params, .. } => assert!(matches!(&params[0].ty, Some(Type::Ref(_)))),
            _ => panic!("Expected Function node"),
        }
    }

    #[test]
    fn test_option_type() {
        match &parse(&lex("fn foo(x: Option<int>) { }"))[0] {
            AstNode::Function { params, .. } => assert!(matches!(&params[0].ty, Some(Type::Option(_)))),
            _ => panic!("Expected Function node"),
        }
    }
}