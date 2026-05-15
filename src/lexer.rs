// src/lexer.rs
// ScrapeC Lexer: Tokenizes ScrapeC source code

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Struct,
    Enum,
    Impl,
    Pub,
    Extern,
    Async,
    Await,
    Use,
    Mod,
    Trait,
    Unsafe,
    Break,
    Continue,
    Loop,
    Match,
    Move,
    Ref,
    True,
    False,
    Some,
    None,
    Ok,
    Err,
    Type,
    As,
    Not,

    // Literals
    Ident(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Eq,
    Ne,
    LessThan,
    GreaterThan,
    Le,
    Ge,
    AndAnd,
    OrOr,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    FatArrow,
    ThinArrow,
    DoubleColon,
    ColonColon,
    DotDot,
    RangeInclusive,

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Colon,
    Dot,
    At,
    Pound,

    // Special
    Eof,
    Unknown(char),
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => { chars.next(); },
            '\n' => { chars.next(); },
            '\r' => { chars.next(); },

            '/' => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' { break; }
                        chars.next();
                    }
                } else if chars.peek() == Some(&'*') {
                    chars.next();
                    let mut depth = 1usize;
                    while depth > 0 {
                        match chars.next() {
                            Some('/') if chars.peek() == Some(&'*') => { depth += 1; chars.next(); },
                            Some('*') if chars.peek() == Some(&'/') => { depth -= 1; chars.next(); },
                            Some(_) => {},
                            None => panic!("Unterminated block comment"),
                        }
                    }
                } else if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::SlashEq);
                } else {
                    tokens.push(Token::Slash);
                }
            },

            '"' => {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => {
                            match chars.next() {
                                Some('n') => s.push('\n'),
                                Some('r') => s.push('\r'),
                                Some('t') => s.push('\t'),
                                Some('\\') => s.push('\\'),
                                Some('"') => s.push('"'),
                                Some('0') => s.push('\0'),
                                Some(c) => { s.push('\\'); s.push(c); },
                                None => panic!("Unterminated string literal"),
                            }
                        },
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => panic!("Unterminated string literal"),
                    }
                }
                tokens.push(Token::String(s));
            },

            '\'' => {
                chars.next();
                let ch = match chars.next() {
                    Some('\\') => {
                        match chars.next() {
                            Some('n') => '\n',
                            Some('r') => '\r',
                            Some('t') => '\t',
                            Some('\\') => '\\',
                            Some('\'') => '\'',
                            Some('0') => '\0',
                            Some(c) => c,
                            None => panic!("Unterminated char literal"),
                        }
                    },
                    Some(c) => c,
                    None => panic!("Unterminated char literal"),
                };
                match chars.next() {
                    Some('\'') => {},
                    _ => panic!("Expected closing quote for char literal"),
                }
                tokens.push(Token::Char(ch));
            },

            '0'..='9' => {
                let mut num_str = String::new();
                let mut has_dot = false;
                while let Some(&ch) = chars.peek() {
                    if ch.is_digit(10) {
                        num_str.push(ch);
                        chars.next();
                    } else if ch == '.' && !has_dot {
                        let mut peek_iter = chars.clone();
                        peek_iter.next();
                        if let Some(&next_ch) = peek_iter.peek() {
                            if next_ch == '.' {
                                break;
                            }
                        }
                        has_dot = true;
                        num_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if has_dot {
                    tokens.push(Token::Float(num_str.parse().unwrap_or(0.0)));
                } else {
                    tokens.push(Token::Int(num_str.parse().unwrap_or(0)));
                }
            },

            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = match ident.as_str() {
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "mut" => Token::Mut,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "for" => Token::For,
                    "in" => Token::In,
                    "return" => Token::Return,
                    "struct" => Token::Struct,
                    "enum" => Token::Enum,
                    "impl" => Token::Impl,
                    "pub" => Token::Pub,
                    "extern" => Token::Extern,
                    "async" => Token::Async,
                    "await" => Token::Await,
                    "use" => Token::Use,
                    "mod" => Token::Mod,
                    "trait" => Token::Trait,
                    "unsafe" => Token::Unsafe,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "loop" => Token::Loop,
                    "match" => Token::Match,
                    "move" => Token::Move,
                    "ref" => Token::Ref,
                    "type" => Token::Type,
                    "as" => Token::As,
                    "not" => Token::Not,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    "Some" => Token::Some,
                    "None" => Token::None,
                    "Ok" => Token::Ok,
                    "Err" => Token::Err,
                    _ => Token::Ident(ident),
                };
                tokens.push(token);
            },

            '=' => {
                chars.next();
                match chars.peek() {
                    Some('=') => { chars.next(); tokens.push(Token::Eq); },
                    Some('>') => { chars.next(); tokens.push(Token::FatArrow); },
                    _ => tokens.push(Token::Assign),
                }
            },
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ne);
                } else {
                    tokens.push(Token::Not);
                }
            },
            '<' => {
                chars.next();
                match chars.peek() {
                    Some('=') => { chars.next(); tokens.push(Token::Le); },
                    Some('<') => { chars.next(); tokens.push(Token::Shl); },
                    _ => tokens.push(Token::LessThan),
                }
            },
            '>' => {
                chars.next();
                match chars.peek() {
                    Some('=') => { chars.next(); tokens.push(Token::Ge); },
                    Some('>') => { chars.next(); tokens.push(Token::Shr); },
                    _ => tokens.push(Token::GreaterThan),
                }
            },
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::AndAnd);
                } else {
                    tokens.push(Token::BitAnd);
                }
            },
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::OrOr);
                } else {
                    tokens.push(Token::BitOr);
                }
            },
            '^' => { chars.next(); tokens.push(Token::BitXor); },
            '~' => { chars.next(); tokens.push(Token::BitNot); },
            '+' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::PlusEq);
                } else {
                    tokens.push(Token::Plus);
                }
            },
            '-' => {
                chars.next();
                match chars.peek() {
                    Some('=') => { chars.next(); tokens.push(Token::MinusEq); },
                    Some('>') => { chars.next(); tokens.push(Token::ThinArrow); },
                    _ => tokens.push(Token::Minus),
                }
            },
            '*' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::StarEq);
                } else {
                    tokens.push(Token::Star);
                }
            },
            '%' => { chars.next(); tokens.push(Token::Percent); },

            '.' => {
                chars.next();
                if chars.peek() == Some(&'.') {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::RangeInclusive);
                    } else {
                        tokens.push(Token::DotDot);
                    }
                } else {
                    tokens.push(Token::Dot);
                }
            },
            ':' => {
                chars.next();
                if chars.peek() == Some(&':') {
                    chars.next();
                    tokens.push(Token::DoubleColon);
                } else {
                    tokens.push(Token::Colon);
                }
            },
            '@' => { chars.next(); tokens.push(Token::At); },
            '#' => { chars.next(); tokens.push(Token::Pound); },

            '(' => { tokens.push(Token::LParen); chars.next(); },
            ')' => { tokens.push(Token::RParen); chars.next(); },
            '{' => { tokens.push(Token::LBrace); chars.next(); },
            '}' => { tokens.push(Token::RBrace); chars.next(); },
            '[' => { tokens.push(Token::LBracket); chars.next(); },
            ']' => { tokens.push(Token::RBracket); chars.next(); },
            ';' => { tokens.push(Token::Semicolon); chars.next(); },
            ',' => { tokens.push(Token::Comma); chars.next(); },

            _ => {
                tokens.push(Token::Unknown(c));
                chars.next();
            }
        }
    }
    tokens.push(Token::Eof);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = lex("fn let mut if else while for return");
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Let);
        assert_eq!(tokens[2], Token::Mut);
        assert_eq!(tokens[3], Token::If);
        assert_eq!(tokens[4], Token::Else);
        assert_eq!(tokens[5], Token::While);
        assert_eq!(tokens[6], Token::For);
        assert_eq!(tokens[7], Token::Return);
    }

    #[test]
    fn test_string_literal() {
        let tokens = lex(r#"let x = "hello";"#);
        assert!(matches!(&tokens[3], Token::String(s) if s == "hello"));
    }

    #[test]
    fn test_escape_string() {
        let tokens = lex(r#"let x = "hello\nworld";"#);
        assert!(matches!(&tokens[3], Token::String(s) if s == "hello\nworld"));
    }

    #[test]
    fn test_char_literal() {
        let tokens = lex("let x = 'a';");
        assert!(matches!(&tokens[3], Token::Char('a')));
    }

    #[test]
    fn test_comments() {
        let source = "// comment\nlet x = 5; // inline\n/* block */ let y = 10;";
        let tokens = lex(source);
        let non_eof: Vec<_> = tokens.iter().filter(|t| **t != Token::Eof).collect();
        assert_eq!(non_eof.len(), 10);
    }

    #[test]
    fn test_comparison_ops() {
        let tokens = lex("a == b != c < d > e <= f >= g");
        assert!(matches!(&tokens[1], Token::Eq));
        assert!(matches!(&tokens[3], Token::Ne));
        assert!(matches!(&tokens[5], Token::LessThan));
        assert!(matches!(&tokens[7], Token::GreaterThan));
        assert!(matches!(&tokens[9], Token::Le));
        assert!(matches!(&tokens[11], Token::Ge));
    }

    #[test]
    fn test_logical_ops() {
        let tokens = lex("a && b || c");
        assert!(matches!(&tokens[1], Token::AndAnd));
        assert!(matches!(&tokens[3], Token::OrOr));
    }

    #[test]
    fn test_compound_assign() {
        let tokens = lex("x += 1; y -= 2; z *= 3; w /= 4;");
        assert!(matches!(&tokens[1], Token::PlusEq));
        assert!(matches!(&tokens[5], Token::MinusEq));
        assert!(matches!(&tokens[9], Token::StarEq));
        assert!(matches!(&tokens[13], Token::SlashEq));
    }

    #[test]
    fn test_bitwise_ops() {
        let tokens = lex("a & b | c ^ d ~ e");
        assert!(matches!(&tokens[1], Token::BitAnd));
        assert!(matches!(&tokens[3], Token::BitOr));
        assert!(matches!(&tokens[5], Token::BitXor));
        assert!(matches!(&tokens[7], Token::BitNot));
    }

    #[test]
    fn test_arrow_tokens() {
        let tokens = lex("fn f() -> i32 { x => y }");
        assert!(matches!(&tokens[4], Token::ThinArrow));
        assert!(matches!(&tokens[8], Token::FatArrow));
    }

    #[test]
    fn test_double_colon() {
        let tokens = lex("std::io::read()");
        assert!(matches!(&tokens[1], Token::DoubleColon));
        assert!(matches!(&tokens[3], Token::DoubleColon));
    }

    #[test]
    fn test_range() {
        let tokens = lex("0..10");
        assert!(matches!(&tokens[0], Token::Int(0)));
        assert!(matches!(&tokens[1], Token::DotDot));
        assert!(matches!(&tokens[2], Token::Int(10)));
    }

    #[test]
    fn test_range_inclusive() {
        let tokens = lex("0..=10");
        assert!(matches!(&tokens[0], Token::Int(0)));
        assert!(matches!(&tokens[1], Token::RangeInclusive));
        assert!(matches!(&tokens[2], Token::Int(10)));
    }

    #[test]
    fn test_struct_enum_keywords() {
        let tokens = lex("struct enum impl pub extern async await mod trait");
        assert_eq!(tokens[0], Token::Struct);
        assert_eq!(tokens[1], Token::Enum);
        assert_eq!(tokens[2], Token::Impl);
        assert_eq!(tokens[3], Token::Pub);
        assert_eq!(tokens[4], Token::Extern);
        assert_eq!(tokens[5], Token::Async);
        assert_eq!(tokens[6], Token::Await);
        assert_eq!(tokens[7], Token::Mod);
        assert_eq!(tokens[8], Token::Trait);
    }

    #[test]
    fn test_not_keyword_and_operator() {
        let tokens = lex("not x != y");
        assert_eq!(tokens[0], Token::Not);
        assert_eq!(tokens[2], Token::Ne);
    }

    #[test]
    fn fn_keyword_not_ident() {
        let tokens = lex("fn_name");
        assert!(matches!(&tokens[0], Token::Ident(s) if s == "fn_name"));
    }
}