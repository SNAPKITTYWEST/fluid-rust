/// Rust Parser - Converts source to HIR (High-level Intermediate Representation)

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Fn,
    Let,
    Mut,
    Return,
    If,
    Else,
    Loop,
    Break,
    Struct,
    Enum,
    // Symbols
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Arrow,
    Colon,
    Semicolon,
    Comma,
    Dot,
    Ampersand,
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
    Question,
    // Literals
    Ident(String),
    IntLit(u64),
    StrLit(String),
    // EOF
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::IntLit(n) => write!(f, "{}", n),
            Token::StrLit(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let ch = self.current_char();

        match ch {
            '{' => {
                self.pos += 1;
                Ok(Token::LeftBrace)
            }
            '}' => {
                self.pos += 1;
                Ok(Token::RightBrace)
            }
            '(' => {
                self.pos += 1;
                Ok(Token::LeftParen)
            }
            ')' => {
                self.pos += 1;
                Ok(Token::RightParen)
            }
            '[' => {
                self.pos += 1;
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.pos += 1;
                Ok(Token::RightBracket)
            }
            ':' => {
                self.pos += 1;
                Ok(Token::Colon)
            }
            ';' => {
                self.pos += 1;
                Ok(Token::Semicolon)
            }
            ',' => {
                self.pos += 1;
                Ok(Token::Comma)
            }
            '.' => {
                self.pos += 1;
                Ok(Token::Dot)
            }
            '&' => {
                self.pos += 1;
                Ok(Token::Ampersand)
            }
            '+' => {
                self.pos += 1;
                Ok(Token::Plus)
            }
            '-' => {
                self.pos += 1;
                if self.current_char() == '>' {
                    self.pos += 1;
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                self.pos += 1;
                Ok(Token::Star)
            }
            '/' => {
                self.pos += 1;
                Ok(Token::Slash)
            }
            '=' => {
                self.pos += 1;
                Ok(Token::Equals)
            }
            '?' => {
                self.pos += 1;
                Ok(Token::Question)
            }
            '"' => self.read_string(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_ident(),
            _ if ch.is_numeric() => self.read_number(),
            _ => Err(format!("Unknown character: {}", ch)),
        }
    }

    fn read_ident(&mut self) -> Result<Token, String> {
        let mut ident = String::new();

        while !self.is_at_end() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            ident.push(self.current_char());
            self.pos += 1;
        }

        Ok(match ident.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "loop" => Token::Loop,
            "break" => Token::Break,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            _ => Token::Ident(ident),
        })
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();

        while !self.is_at_end() && self.current_char().is_numeric() {
            num_str.push(self.current_char());
            self.pos += 1;
        }

        num_str.parse::<u64>()
            .map(Token::IntLit)
            .map_err(|e| format!("Invalid number: {}", e))
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.pos += 1; // skip opening quote
        let mut string = String::new();

        while !self.is_at_end() && self.current_char() != '"' {
            string.push(self.current_char());
            self.pos += 1;
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        self.pos += 1; // skip closing quote
        Ok(Token::StrLit(string))
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current_char().is_whitespace() {
            self.pos += 1;
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.pos]
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        ty: Option<Type>,
        init: Option<Expression>,
    },
    Return {
        value: Option<Expression>,
    },
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Ident(String),
    IntLit(u64),
    StrLit(String),
    Call {
        func: String,
        args: Vec<Expression>,
    },
    BinaryOp {
        op: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub refinement: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Unit,
    Bool,
    Usize,
    U8,
    I32,
    Str,
    Reference {
        mutable: bool,
    },
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_function(&mut self) -> Result<FunctionDef, String> {
        self.expect(Token::Fn)?;

        let name = self.parse_ident()?;

        self.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RightParen)?;

        let return_type = if self.check(&Token::Arrow) {
            self.pos += 1;
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::LeftBrace)?;
        let body = self.parse_statements()?;
        self.expect(Token::RightBrace)?;

        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Type)>, String> {
        let mut params = Vec::new();

        while !self.check(&Token::RightParen) {
            let name = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;

            params.push((name, ty));

            if !self.check(&Token::RightParen) {
                self.expect(Token::Comma)?;
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let mut kind = match &self.current_token() {
            Token::Ident(s) if s == "usize" => TypeKind::Usize,
            Token::Ident(s) if s == "u8" => TypeKind::U8,
            Token::Ident(s) if s == "i32" => TypeKind::I32,
            Token::Ident(s) if s == "str" => TypeKind::Str,
            Token::Ident(s) if s == "bool" => TypeKind::Bool,
            Token::Ampersand => {
                self.pos += 1;
                let mutable = if self.check_ident("mut") {
                    self.pos += 1;
                    true
                } else {
                    false
                };
                TypeKind::Reference { mutable }
            }
            _ => return Err(format!("Expected type, got {}", self.current_token())),
        };

        if !matches!(kind, TypeKind::Reference { .. }) {
            self.pos += 1;
        }

        let refinement = if self.check(&Token::LeftBrace) {
            self.pos += 1;
            let mut pred = String::new();
            while !self.check(&Token::RightBrace) {
                pred.push_str(&format!("{} ", self.current_token()));
                self.pos += 1;
            }
            self.expect(Token::RightBrace)?;
            Some(pred.trim().to_string())
        } else {
            None
        };

        Ok(Type { kind, refinement })
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.check(&Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.current_token() {
            Token::Let => {
                self.pos += 1;
                let name = self.parse_ident()?;

                let ty = if self.check(&Token::Colon) {
                    self.pos += 1;
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let init = if self.check(&Token::Equals) {
                    self.pos += 1;
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                self.expect(Token::Semicolon)?;

                Ok(Statement::Let { name, ty, init })
            }
            Token::Return => {
                self.pos += 1;
                let value = if !self.check(&Token::Semicolon) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Statement::Return { value })
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_call_or_ident()
    }

    fn parse_call_or_ident(&mut self) -> Result<Expression, String> {
        let expr = match &self.current_token() {
            Token::Ident(name) => {
                let name = name.clone();
                self.pos += 1;

                if self.check(&Token::LeftParen) {
                    self.pos += 1;
                    let args = self.parse_args()?;
                    self.expect(Token::RightParen)?;
                    Expression::Call { func: name, args }
                } else {
                    Expression::Ident(name)
                }
            }
            Token::IntLit(n) => {
                let n = *n;
                self.pos += 1;
                Expression::IntLit(n)
            }
            Token::StrLit(s) => {
                let s = s.clone();
                self.pos += 1;
                Expression::StrLit(s)
            }
            _ => return Err(format!("Unexpected token in expression: {}", self.current_token())),
        };

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>, String> {
        let mut args = Vec::new();

        while !self.check(&Token::RightParen) {
            args.push(self.parse_expression()?);
            if !self.check(&Token::RightParen) {
                self.expect(Token::Comma)?;
            }
        }

        Ok(args)
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        match &self.current_token() {
            Token::Ident(s) => {
                let s = s.clone();
                self.pos += 1;
                Ok(s)
            }
            _ => Err(format!("Expected identifier, got {}", self.current_token())),
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.check(&token) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {}", token, self.current_token()))
        }
    }

    fn check(&self, token: &Token) -> bool {
        match (token, &self.current_token()) {
            (Token::Fn, Token::Fn) => true,
            (Token::Let, Token::Let) => true,
            (Token::LeftBrace, Token::LeftBrace) => true,
            (Token::RightBrace, Token::RightBrace) => true,
            (Token::LeftParen, Token::LeftParen) => true,
            (Token::RightParen, Token::RightParen) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::Semicolon, Token::Semicolon) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Arrow, Token::Arrow) => true,
            (Token::Equals, Token::Equals) => true,
            (Token::RightBracket, Token::RightBracket) => true,
            _ => false,
        }
    }

    fn check_ident(&self, s: &str) -> bool {
        matches!(&self.current_token(), Token::Ident(name) if name == s)
    }

    fn current_token(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple_function() {
        let mut lexer = Lexer::new("fn main() { }");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::Fn)));
    }

    #[test]
    fn test_parse_empty_function() {
        let mut lexer = Lexer::new("fn main() { }").tokenize().unwrap();
        let mut parser = Parser::new(lexer);
        let func = parser.parse_function().unwrap();
        assert_eq!(func.name, "main");
    }
}
