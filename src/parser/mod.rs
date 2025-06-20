use crate::ast::*;
use crate::constants::HER_KEY_WORDS;
use crate::lexer::Lexer;
use crate::token::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { want: Option<Token>, got: Token },
    HerUnexpectedToken { got: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { want: w, got: g } => match w {
                Some(w) => write!(
                    f,
                    "啊啊啊啊啊啊啊啊啊啊啊啊 Unexpected Token: expected {w:?}, got {g:?}"
                ),
                None => write!(
                    f,
                    "啊啊啊啊啊啊啊啊啊啊啊啊 Unexpected Token: no prefix rule for {g:?}"
                ),
            },
            ParseError::HerUnexpectedToken { got: g } => {
                write!(f, "啊啊啊啊啊啊啊啊啊啊啊啊 SyntaxError: {g:?}")
            }
        }
    }
}

pub type ParseErrors = Vec<ParseError>;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    next_token: Token,
    errors: ParseErrors,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::Eof,
            next_token: Token::Eof,
            errors: vec![],
        };

        parser.bump();
        parser.bump();

        parser
    }

    fn token_to_precedence(tok: &Token) -> Precedence {
        match tok {
            Token::Equal | Token::NotEqual => Precedence::Equals,
            Token::LessThan | Token::LessThanEqual => Precedence::LessGreater,
            Token::GreaterThan | Token::GreaterThanEqual => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Asterisk => Precedence::Product,
            Token::Lbracket => Precedence::Index,
            Token::Dot => Precedence::Index,
            Token::Lparen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    pub fn get_errors(&mut self) -> ParseErrors {
        self.errors.clone()
    }

    fn bump(&mut self) {
        // FIXME: Clearly unnecessary clone
        self.current_token = self.next_token.clone();
        self.next_token = self.lexer.next_token();
    }

    fn current_token_is(&mut self, tok: Token) -> bool {
        self.current_token == tok
    }

    fn next_token_is(&mut self, tok: &Token) -> bool {
        self.next_token == *tok
    }

    fn expect_next_token(&mut self, tok: Token) -> bool {
        if self.next_token_is(&tok) {
            self.bump();
            true
        } else {
            self.error_next_token(tok);
            false
        }
    }

    fn current_token_precedence(&mut self) -> Precedence {
        Self::token_to_precedence(&self.current_token)
    }

    fn next_token_precedence(&mut self) -> Precedence {
        Self::token_to_precedence(&self.next_token)
    }

    fn error_next_token(&mut self, tok: Token) {
        self.errors.push(ParseError::UnexpectedToken {
            want: Some(tok),
            got: self.next_token.clone(),
        });
    }

    fn error_no_prefix_parser(&mut self) {
        self.errors.push(ParseError::UnexpectedToken {
            want: None,
            got: self.next_token.clone(),
        });
    }

    pub fn parse(&mut self) -> Program {
        let mut program: Program = vec![];

        while !self.current_token_is(Token::Eof) {
            match self.parse_stmt() {
                Some(stmt) => program.push(stmt),
                None => {}
            }
            self.bump();
        }

        program
    }

    fn parse_block_stmt(&mut self) -> BlockStmt {
        self.bump();

        let mut block = vec![];

        while !self.current_token_is(Token::Rbrace) {
            if self.current_token_is(Token::Eof) {
                self.error_next_token(Token::Rbrace);
                return block;
            }
            match self.parse_stmt() {
                Some(stmt) => block.push(stmt),
                None => {}
            }
            self.bump();
        }

        block
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Let => self.parse_let_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Blank => Some(Stmt::Blank),
            Token::Break => self.parse_break_stmt(),
            Token::Continue => self.parse_continue_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        match &self.next_token {
            Token::Ident(_) => self.bump(),
            _ => return None,
        };

        let name = self.parse_ident()?;

        if !self.expect_next_token(Token::Assign) {
            return None;
        }

        // 女性是不能被定义滴
        if HER_KEY_WORDS.contains(&name.0.as_str()) {
            self.errors.push(ParseError::HerUnexpectedToken {
                got: format!("女性是不能被定义的！！！"),
            });
            return None;
        };

        self.bump();

        let expr = self.parse_expr(Precedence::Lowest)?;

        if self.next_token_is(&Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Let(name, expr))
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.bump();

        let expr = self.parse_expr(Precedence::Lowest)?;

        if self.next_token_is(&Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Return(expr))
    }

    fn parse_break_stmt(&mut self) -> Option<Stmt> {
        self.bump();

        if self.next_token_is(&Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Break)
    }

    fn parse_continue_stmt(&mut self) -> Option<Stmt> {
        self.bump();

        if self.next_token_is(&Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Continue)
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => {
                if self.next_token_is(&Token::Semicolon) {
                    self.bump();
                }
                Some(Stmt::Expr(expr))
            }
            None => None,
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        // prefix
        let mut left = match self.current_token {
            Token::Ident(_) => self.parse_ident_expr(),
            Token::Int(_) => self.parse_int_expr(),
            Token::String(_) => self.parse_string_expr(),
            Token::Bool(_) => self.parse_bool_expr(),
            Token::Lbracket => self.parse_array_expr(),
            Token::Lbrace => self.parse_hash_expr(),
            Token::Bang | Token::Minus | Token::Plus => self.parse_prefix_expr(),
            Token::Lparen => self.parse_grouped_expr(),
            Token::If => self.parse_if_expr(),
            Token::While => self.parse_while_expr(),
            Token::Func => self.parse_func_expr(),
            _ => {
                self.error_no_prefix_parser();
                return None;
            }
        };

        // infix
        while !self.next_token_is(&Token::Semicolon) && precedence < self.next_token_precedence() {
            match self.next_token {
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::LessThanEqual
                | Token::GreaterThan
                | Token::GreaterThanEqual => {
                    self.bump();
                    left = self.parse_infix_expr(left.unwrap());
                }
                Token::Lbracket => {
                    self.bump();
                    left = self.parse_index_expr(left.unwrap());
                }
                Token::Dot => {
                    self.bump();
                    left = self.parse_dot_access_expr(left.unwrap());
                }
                Token::Lparen => {
                    self.bump();
                    left = self.parse_call_expr(left.unwrap());
                }
                _ => return left,
            }
        }

        left
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        match self.current_token {
            Token::Ident(ref mut ident) => Some(Ident(ident.clone())),
            _ => None,
        }
    }

    fn parse_ident_expr(&mut self) -> Option<Expr> {
        self.parse_ident().map(Expr::Ident)
    }

    fn parse_int_expr(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Int(ref mut int) => Some(Expr::Literal(Literal::Int(*int))),
            _ => None,
        }
    }

    fn parse_string_expr(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::String(ref mut s) => Some(Expr::Literal(Literal::String(s.clone()))),
            _ => None,
        }
    }

    fn parse_bool_expr(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Bool(value) => Some(Expr::Literal(Literal::Bool(value))),
            _ => None,
        }
    }

    fn parse_array_expr(&mut self) -> Option<Expr> {
        self.parse_expr_list(Token::Rbracket)
            .map(|list| Expr::Literal(Literal::Array(list)))
    }

    fn parse_hash_expr(&mut self) -> Option<Expr> {
        let mut pairs = Vec::new();

        while !self.next_token_is(&Token::Rbrace) {
            self.bump();

            let key = self.parse_expr(Precedence::Lowest)?;

            if !self.expect_next_token(Token::Colon) {
                return None;
            }

            self.bump();

            let value = self.parse_expr(Precedence::Lowest)?;

            pairs.push((key, value));

            if !self.next_token_is(&Token::Rbrace) && !self.expect_next_token(Token::Comma) {
                return None;
            }
        }

        if !self.expect_next_token(Token::Rbrace) {
            return None;
        }

        Some(Expr::Literal(Literal::Hash(pairs)))
    }

    fn parse_expr_list(&mut self, end: Token) -> Option<Vec<Expr>> {
        let mut list = vec![];

        if self.next_token_is(&end) {
            self.bump();
            return Some(list);
        }

        self.bump();

        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => list.push(expr),
            None => return None,
        }

        while self.next_token_is(&Token::Comma) {
            self.bump();
            self.bump();

            match self.parse_expr(Precedence::Lowest) {
                Some(expr) => list.push(expr),
                None => return None,
            }
        }

        if !self.expect_next_token(end) {
            return None;
        }

        Some(list)
    }

    fn parse_prefix_expr(&mut self) -> Option<Expr> {
        let prefix = match self.current_token {
            Token::Bang => Prefix::Not,
            Token::Minus => Prefix::Minus,
            Token::Plus => Prefix::Plus,
            _ => return None,
        };

        self.bump();

        self.parse_expr(Precedence::Prefix)
            .map(|expr| Expr::Prefix(prefix, Box::new(expr)))
    }

    fn parse_infix_expr(&mut self, left: Expr) -> Option<Expr> {
        let infix = match self.current_token {
            Token::Plus => Infix::Plus,
            Token::Minus => Infix::Minus,
            Token::Slash => Infix::Divide,
            Token::Asterisk => Infix::Multiply,
            Token::Equal => Infix::Equal,
            Token::NotEqual => Infix::NotEqual,
            Token::LessThan => Infix::LessThan,
            Token::LessThanEqual => Infix::LessThanEqual,
            Token::GreaterThan => Infix::GreaterThan,
            Token::GreaterThanEqual => Infix::GreaterThanEqual,
            _ => return None,
        };

        let precedence = self.current_token_precedence();

        self.bump();

        self.parse_expr(precedence)
            .map(|expr| Expr::Infix(infix, Box::new(left), Box::new(expr)))
    }

    fn parse_index_expr(&mut self, left: Expr) -> Option<Expr> {
        self.bump();

        let index = self.parse_expr(Precedence::Lowest)?;

        if !self.expect_next_token(Token::Rbracket) {
            return None;
        }

        Some(Expr::Index(Box::new(left), Box::new(index)))
    }

    fn parse_dot_access_expr(&mut self, left: Expr) -> Option<Expr> {
        self.bump();

        self.parse_ident().map(|Ident(str)| {
            Expr::Index(
                Box::new(left),
                Box::new(Expr::Literal(Literal::String(str))),
            )
        })
    }

    fn parse_grouped_expr(&mut self) -> Option<Expr> {
        self.bump();

        let expr = self.parse_expr(Precedence::Lowest);

        if !self.expect_next_token(Token::Rparen) {
            None
        } else {
            expr
        }
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        if !self.expect_next_token(Token::Lparen) {
            return None;
        }

        self.bump();

        let cond = self.parse_expr(Precedence::Lowest)?;

        if !self.expect_next_token(Token::Rparen) || !self.expect_next_token(Token::Lbrace) {
            return None;
        }

        let consequence = self.parse_block_stmt();
        let mut alternative = None;

        if self.next_token_is(&Token::Else) {
            self.bump();

            if !self.expect_next_token(Token::Lbrace) {
                return None;
            }

            alternative = Some(self.parse_block_stmt());
        }

        Some(Expr::If {
            cond: Box::new(cond),
            consequence,
            alternative,
        })
    }

    fn parse_while_expr(&mut self) -> Option<Expr> {
        if !self.expect_next_token(Token::Lparen) {
            return None;
        }

        self.bump();

        let cond = self.parse_expr(Precedence::Lowest)?;

        if !self.expect_next_token(Token::Rparen) || !self.expect_next_token(Token::Lbrace) {
            return None;
        }

        let consequence = self.parse_block_stmt();

        Some(Expr::While {
            cond: Box::new(cond),
            consequence,
        })
    }

    fn parse_func_expr(&mut self) -> Option<Expr> {
        if !self.expect_next_token(Token::Lparen) {
            return None;
        }

        let params = self.parse_func_params()?;

        if !self.expect_next_token(Token::Lbrace) {
            return None;
        }

        Some(Expr::Func {
            params,
            body: self.parse_block_stmt(),
        })
    }

    fn parse_func_params(&mut self) -> Option<Vec<Ident>> {
        let mut params = vec![];

        if self.next_token_is(&Token::Rparen) {
            self.bump();
            return Some(params);
        }

        self.bump();

        match self.parse_ident() {
            Some(ident) => params.push(ident),
            None => return None,
        };

        while self.next_token_is(&Token::Comma) {
            self.bump();
            self.bump();

            match self.parse_ident() {
                Some(ident) => params.push(ident),
                None => return None,
            };
        }

        if !self.expect_next_token(Token::Rparen) {
            return None;
        }

        Some(params)
    }

    fn parse_call_expr(&mut self, func: Expr) -> Option<Expr> {
        let args = self.parse_expr_list(Token::Rparen)?;

        Some(Expr::Call {
            func: Box::new(func),
            args,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn check_parse_errors(parser: &mut Parser) {
        let errors = parser.get_errors();

        if errors.is_empty() {
            return;
        }

        println!("\n");

        println!("parser has {} errors", errors.len());

        for err in errors {
            println!("parse error: {:?}", err);
        }

        println!("\n");

        panic!("failed");
    }

    #[test]
    fn test_blank() {
        let input = r#"
1000;

1000;


1000;

if (x) {

    x;

}
        "#;

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![
                Stmt::Expr(Expr::Literal(Literal::Int(1000))),
                Stmt::Blank,
                Stmt::Expr(Expr::Literal(Literal::Int(1000))),
                Stmt::Blank,
                Stmt::Blank,
                Stmt::Expr(Expr::Literal(Literal::Int(1000))),
                Stmt::Blank,
                Stmt::Expr(Expr::If {
                    cond: Box::new(Expr::Ident(Ident(String::from("x")))),
                    consequence: vec![
                        Stmt::Blank,
                        Stmt::Expr(Expr::Ident(Ident(String::from("x")))),
                        Stmt::Blank,
                    ],
                    alternative: None,
                }),
            ],
            program,
        );
    }

    #[test]
    fn test_let_stmt() {
        let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
        "#;

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![
                Stmt::Let(Ident(String::from("x")), Expr::Literal(Literal::Int(5))),
                Stmt::Let(Ident(String::from("y")), Expr::Literal(Literal::Int(10))),
                Stmt::Let(
                    Ident(String::from("foobar")),
                    Expr::Literal(Literal::Int(838383)),
                ),
            ],
            program,
        );
    }

    #[test]
    fn test_return_stmt() {
        let input = r#"
return 5;
return 10;
return 993322;
        "#;

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![
                Stmt::Return(Expr::Literal(Literal::Int(5))),
                Stmt::Return(Expr::Literal(Literal::Int(10))),
                Stmt::Return(Expr::Literal(Literal::Int(993322))),
            ],
            program,
        );
    }

    #[test]
    fn test_ident_expr() {
        let input = "foobar;";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Ident(Ident(String::from("foobar"))))],
            program,
        );
    }

    #[test]
    fn test_integer_literal_expr() {
        let input = "5;";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(vec![Stmt::Expr(Expr::Literal(Literal::Int(5)))], program,);
    }

    #[test]
    fn test_string_literal_expr() {
        let input = "\"herllo world\";";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Literal(Literal::String(String::from(
                "herllo world",
            ))))],
            program,
        );
    }

    #[test]
    fn test_boolean_literal_expr() {
        let tests = vec![
            ("true;", Stmt::Expr(Expr::Literal(Literal::Bool(true)))),
            ("false;", Stmt::Expr(Expr::Literal(Literal::Bool(false)))),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_array_literal_expr() {
        let input = "[1, 2 * 2, 3 + 3]";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Literal(Literal::Array(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Literal(Literal::Int(2))),
                    Box::new(Expr::Literal(Literal::Int(2))),
                ),
                Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Literal(Literal::Int(3))),
                    Box::new(Expr::Literal(Literal::Int(3))),
                ),
            ])))],
            program,
        );
    }

    #[test]
    fn test_hash_literal_expr() {
        let tests = vec![
            ("{}", Stmt::Expr(Expr::Literal(Literal::Hash(vec![])))),
            (
                "{\"one\": 1, \"two\": 2, \"three\": 3}",
                Stmt::Expr(Expr::Literal(Literal::Hash(vec![
                    (
                        Expr::Literal(Literal::String(String::from("one"))),
                        Expr::Literal(Literal::Int(1)),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("two"))),
                        Expr::Literal(Literal::Int(2)),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("three"))),
                        Expr::Literal(Literal::Int(3)),
                    ),
                ]))),
            ),
            (
                "{\"one\": 0 + 1, \"two\": 10 - 8, \"three\": 15 / 5}",
                Stmt::Expr(Expr::Literal(Literal::Hash(vec![
                    (
                        Expr::Literal(Literal::String(String::from("one"))),
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(0))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("two"))),
                        Expr::Infix(
                            Infix::Minus,
                            Box::new(Expr::Literal(Literal::Int(10))),
                            Box::new(Expr::Literal(Literal::Int(8))),
                        ),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("three"))),
                        Expr::Infix(
                            Infix::Divide,
                            Box::new(Expr::Literal(Literal::Int(15))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                ]))),
            ),
            (
                "{key: \"value\"}",
                Stmt::Expr(Expr::Literal(Literal::Hash(vec![(
                    Expr::Ident(Ident(String::from("key"))),
                    Expr::Literal(Literal::String(String::from("value"))),
                )]))),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_index_expr() {
        let input = "myArray[1 + 1]";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Index(
                Box::new(Expr::Ident(Ident(String::from("myArray")))),
                Box::new(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Literal(Literal::Int(1))),
                    Box::new(Expr::Literal(Literal::Int(1))),
                )),
            ))],
            program
        );
    }

    #[test]
    fn test_dot_access_expr() {
        let input = "myHash.key";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Index(
                Box::new(Expr::Ident(Ident(String::from("myHash")))),
                Box::new(Expr::Literal(Literal::String(String::from("key")))),
            ))],
            program
        );
    }

    #[test]
    fn test_prefix_expr() {
        let tests = vec![
            (
                "!5;",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Not,
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "-15;",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Minus,
                    Box::new(Expr::Literal(Literal::Int(15))),
                )),
            ),
            (
                "+15;",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Plus,
                    Box::new(Expr::Literal(Literal::Int(15))),
                )),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_infix_expr() {
        let tests = vec![
            (
                "5 + 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 - 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Minus,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 * 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 / 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 > 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::GreaterThan,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 < 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::LessThan,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 == 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 != 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::NotEqual,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 >= 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::GreaterThanEqual,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 <= 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::LessThanEqual,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_if_expr() {
        let input = "if (x < y) { x }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::If {
                cond: Box::new(Expr::Infix(
                    Infix::LessThan,
                    Box::new(Expr::Ident(Ident(String::from("x")))),
                    Box::new(Expr::Ident(Ident(String::from("y")))),
                )),
                consequence: vec![Stmt::Expr(Expr::Ident(Ident(String::from("x"))))],
                alternative: None,
            })],
            program,
        );
    }

    #[test]
    fn test_if_else_expr() {
        let input = "if (x < y) { x } else { y }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::If {
                cond: Box::new(Expr::Infix(
                    Infix::LessThan,
                    Box::new(Expr::Ident(Ident(String::from("x")))),
                    Box::new(Expr::Ident(Ident(String::from("y")))),
                )),
                consequence: vec![Stmt::Expr(Expr::Ident(Ident(String::from("x"))))],
                alternative: Some(vec![Stmt::Expr(Expr::Ident(Ident(String::from("y"))))]),
            })],
            program,
        );
    }

    #[test]
    fn test_func_expr() {
        let input = "fn(x, y) { x + y; }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Func {
                params: vec![Ident(String::from("x")), Ident(String::from("y"))],
                body: vec![Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Ident(Ident(String::from("x")))),
                    Box::new(Expr::Ident(Ident(String::from("y")))),
                ))],
            })],
            program,
        );
    }

    #[test]
    fn test_func_params() {
        let tests = vec![
            ("fn() {};", vec![]),
            ("fn(x) {};", vec![Ident(String::from("x"))]),
            (
                "fn(x, y, z) {};",
                vec![
                    Ident(String::from("x")),
                    Ident(String::from("y")),
                    Ident(String::from("z")),
                ],
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(
                vec![Stmt::Expr(Expr::Func {
                    params: expect,
                    body: vec![],
                })],
                program,
            );
        }
    }

    #[test]
    fn test_call_expr() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Call {
                func: Box::new(Expr::Ident(Ident(String::from("add")))),
                args: vec![
                    Expr::Literal(Literal::Int(1)),
                    Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Literal(Literal::Int(2))),
                        Box::new(Expr::Literal(Literal::Int(3))),
                    ),
                    Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(4))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    ),
                ],
            })],
            program,
        );
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            (
                "-a * b",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Prefix(
                        Prefix::Minus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("b")))),
                )),
            ),
            (
                "!-a",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Not,
                    Box::new(Expr::Prefix(
                        Prefix::Minus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                    )),
                )),
            ),
            (
                "a + b + c",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a + b - c",
                Stmt::Expr(Expr::Infix(
                    Infix::Minus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a * b * c",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a * b / c",
                Stmt::Expr(Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a + b / c",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Ident(Ident(String::from("a")))),
                    Box::new(Expr::Infix(
                        Infix::Divide,
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                        Box::new(Expr::Ident(Ident(String::from("c")))),
                    )),
                )),
            ),
            (
                "a + b * c + d / e - f",
                Stmt::Expr(Expr::Infix(
                    Infix::Minus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Infix(
                                Infix::Multiply,
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Ident(Ident(String::from("c")))),
                            )),
                        )),
                        Box::new(Expr::Infix(
                            Infix::Divide,
                            Box::new(Expr::Ident(Ident(String::from("d")))),
                            Box::new(Expr::Ident(Ident(String::from("e")))),
                        )),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("f")))),
                )),
            ),
            (
                "5 > 4 == 3 < 4",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::GreaterThan,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::LessThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 < 4 != 3 > 4",
                Stmt::Expr(Expr::Infix(
                    Infix::NotEqual,
                    Box::new(Expr::Infix(
                        Infix::LessThan,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::GreaterThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 >= 4 == 3 <= 4",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::GreaterThanEqual,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::LessThanEqual,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 <= 4 != 3 >= 4",
                Stmt::Expr(Expr::Infix(
                    Infix::NotEqual,
                    Box::new(Expr::Infix(
                        Infix::LessThanEqual,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::GreaterThanEqual,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(4))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        )),
                    )),
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        )),
                        Box::new(Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(4))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        )),
                    )),
                )),
            ),
            ("true", Stmt::Expr(Expr::Literal(Literal::Bool(true)))),
            ("false", Stmt::Expr(Expr::Literal(Literal::Bool(false)))),
            (
                "3 > 5 == false",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::GreaterThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expr::Literal(Literal::Bool(false))),
                )),
            ),
            (
                "3 < 5 == true",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::LessThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expr::Literal(Literal::Bool(true))),
                )),
            ),
            (
                "1 + (2 + 3) + 4",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(1))),
                        Box::new(Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(2))),
                            Box::new(Expr::Literal(Literal::Int(3))),
                        )),
                    )),
                    Box::new(Expr::Literal(Literal::Int(4))),
                )),
            ),
            (
                "(5 + 5) * 2",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expr::Literal(Literal::Int(2))),
                )),
            ),
            (
                "2 / (5 + 5)",
                Stmt::Expr(Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Literal(Literal::Int(2))),
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                )),
            ),
            (
                "-(5 + 5)",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Minus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                )),
            ),
            (
                "!(true == true)",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Not,
                    Box::new(Expr::Infix(
                        Infix::Equal,
                        Box::new(Expr::Literal(Literal::Bool(true))),
                        Box::new(Expr::Literal(Literal::Bool(true))),
                    )),
                )),
            ),
            (
                "a + add(b * c) + d",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Call {
                            func: Box::new(Expr::Ident(Ident(String::from("add")))),
                            args: vec![Expr::Infix(
                                Infix::Multiply,
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Ident(Ident(String::from("c")))),
                            )],
                        }),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("d")))),
                )),
            ),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident(Ident(String::from("add")))),
                    args: vec![
                        Expr::Ident(Ident(String::from("a"))),
                        Expr::Ident(Ident(String::from("b"))),
                        Expr::Literal(Literal::Int(1)),
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(2))),
                            Box::new(Expr::Literal(Literal::Int(3))),
                        ),
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(4))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                        Expr::Call {
                            func: Box::new(Expr::Ident(Ident(String::from("add")))),
                            args: vec![
                                Expr::Literal(Literal::Int(6)),
                                Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Literal(Literal::Int(7))),
                                    Box::new(Expr::Literal(Literal::Int(8))),
                                ),
                            ],
                        },
                    ],
                }),
            ),
            (
                "add(a + b + c * d / f + g)",
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident(Ident(String::from("add")))),
                    args: vec![Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Infix(
                                Infix::Plus,
                                Box::new(Expr::Ident(Ident(String::from("a")))),
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                            )),
                            Box::new(Expr::Infix(
                                Infix::Divide,
                                Box::new(Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Ident(Ident(String::from("c")))),
                                    Box::new(Expr::Ident(Ident(String::from("d")))),
                                )),
                                Box::new(Expr::Ident(Ident(String::from("f")))),
                            )),
                        )),
                        Box::new(Expr::Ident(Ident(String::from("g")))),
                    )],
                }),
            ),
            (
                "a * [1, 2, 3, 4][b * c] * d",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Index(
                            Box::new(Expr::Literal(Literal::Array(vec![
                                Expr::Literal(Literal::Int(1)),
                                Expr::Literal(Literal::Int(2)),
                                Expr::Literal(Literal::Int(3)),
                                Expr::Literal(Literal::Int(4)),
                            ]))),
                            Box::new(Expr::Infix(
                                Infix::Multiply,
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Ident(Ident(String::from("c")))),
                            )),
                        )),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("d")))),
                )),
            ),
            (
                "add(a * b[2], b[1], 2 * [1, 2][1])",
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident(Ident(String::from("add")))),
                    args: vec![
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Index(
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Literal(Literal::Int(2))),
                            )),
                        ),
                        Expr::Index(
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(2))),
                            Box::new(Expr::Index(
                                Box::new(Expr::Literal(Literal::Array(vec![
                                    Expr::Literal(Literal::Int(1)),
                                    Expr::Literal(Literal::Int(2)),
                                ]))),
                                Box::new(Expr::Literal(Literal::Int(1))),
                            )),
                        ),
                    ],
                }),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }
}
