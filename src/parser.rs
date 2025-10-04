use crate::ast::{Ast, BinaryOperators, Expr, Stmt, UnaryOperators};
use crate::lexer::Token;

/**
 * Parser is responsable to evaluate token and build the correct AST
 *
 */
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
        }
    }

    pub fn parse_tokens(&mut self) {
        let mut stmts: Vec<Stmt> = Vec::new();
        loop {
            if let Some(token) = self.get_token() {
                let stmt = match token {
                    Token::Local => self.evaluate_local(),
                    Token::Integer(int) => self.evaluate_integer(int),
                    Token::Float(float) => self.evaluate_float(float),
                    Token::String(str) => self.evaluate_string(str),
                    Token::Boolean(bool) => self.evaluate_bool(bool),
                    Token::Identifier(identifier) => self.evaluate_identifier(identifier),
                    Token::Nil => self.evaluate_unary(),
                    Token::Print => self.evaluate_print(None),
                    Token::If => self.evaluate_if(),
                    Token::Assign => self.evaluate_assign(),
                    Token::End => Stmt::ExprStmt(self.evaluate_end()),
                    Token::For => self.evaluate_for(),
                    Token::Do => todo!(),
                    Token::While => todo!(),
                    Token::Repeat => todo!(),
                    Token::Break => todo!(),
                    Token::Until => todo!(),
                    Token::Function => todo!(),
                    Token::DoubleQuote => todo!(),
                    Token::Plus => todo!(),
                    Token::Minus => todo!(),
                    Token::Divider => todo!(),
                    Token::Modulo => todo!(),
                    Token::Multiplier => todo!(),
                    Token::LParen => todo!(),
                    Token::RParen => todo!(),
                    Token::LBrace => todo!(),
                    Token::RBrace => todo!(),
                    Token::Gt => todo!(),
                    Token::Eq => todo!(),
                    Token::Lt => Stmt::BinaryOperatorsStmt(BinaryOperators::Lt),
                    Token::Not => todo!(),
                    Token::NotEq => todo!(),
                    Token::GtEq => todo!(),
                    Token::LtEq => todo!(),
                    Token::Return => todo!(),
                    Token::Eof => Stmt::Eof(),
                    _ => todo!(),
                };

                stmts.push(stmt);
                self.advance();
            } else {
                println!("stmt {:?}", stmts);
                break;
            }
        }
    }

    fn get_token(&mut self) -> Option<Token> {
        self.tokens.iter().nth(self.pos).cloned()
    }

    fn get_local_identifier(&mut self) -> String {
        self.advance();
        if let Some(token) = self.get_token() {
            match token {
                Token::Identifier(identifier) => identifier,
                _ => {
                    panic!("Not found any string in Identifier")
                }
            }
        } else {
            panic!("Not found local identifier")
        }
    }

    fn evaluate_print(&mut self, value: Option<String>) -> Stmt {
        self.advance();
        if let Some(token) = self.get_token() {
            match token {
                Token::LParen => {
                    self.advance();
                    self.evaluate_print(None)
                }
                Token::RParen => {
                    if let Some(print_value) = value {
                        self.advance();
                        return Stmt::Print {
                            value: Expr::String(print_value),
                        };
                    } else {
                        panic!("Print is empty");
                    }
                }
                Token::String(str) => self.evaluate_print(Some(str)),
                Token::Integer(int) => self.evaluate_print(Some(int.to_string())),
                Token::Float(float) => self.evaluate_print(Some(float.to_string())),
                _ => panic!("Print requires a string, float or integer!"),
            }
        } else {
            panic!("Print is empty")
        }
    }

    fn evaluate_if(&mut self) -> Stmt {
        self.advance();
        Stmt::If {
            condition: self.evaluate_condition(),
            then_block: Some(Box::new(self.evaluate_then())),
            else_block: Some(Box::new(self.evaluate_else())),
            end: self.evaluate_end(),
        }
    }

    fn evaluate_end(&mut self) -> Expr {
        return Expr::End;
    }

    fn evaluate_condition(&mut self) -> Expr {
        let left_token = match self.evaluate_base_token() {
            Some(expr) => expr,
            None => panic!("Expected expression in if"),
        };

        self.advance();
        if let Some(binary_operator) = self.evaluate_binary_operators() {
            self.advance();

            let right_token = match self.evaluate_base_token() {
                Some(expr) => expr,
                None => panic!("Expected expression after a binary operator"),
            };

            Expr::Binary {
                op: binary_operator,
                left: Box::new(left_token),
                right: Box::new(right_token),
            }
        } else {
            return left_token;
        }
    }

    fn evaluate_then(&mut self) -> Stmt {
        self.advance();
        return self.evaluate_stmt();
    }

    fn evaluate_else(&mut self) -> Option<Stmt> {
        let else_token = self.get_token();
        match else_token {
            Some(Token::Else) => {
                self.advance();
                Some(self.evaluate_stmt())
            },
            _ => None
        }
    }

    fn evaluate_stmt(&mut self) -> Stmt {
        let mut stmt: Vec<Stmt> = Vec::new();

        while let Some(token) = self.get_token() {
            match token {
                Token::If => stmt.push(self.evaluate_if()),
                Token::Else | Token::End => break,
                Token::Assign => stmt.push(self.evaluate_assign()),
                Token::For => stmt.push(self.evaluate_for()),
                Token::Break => stmt.push(self.evaluate_break()),
                _ => {
                    if let Some(base_token) = self.evaluate_base_token() {
                        stmt.push(Stmt::ExprStmt(base_token))
                    }
                }
            }
            self.advance();
        }

        return Stmt::Block(stmt);
    }

    fn evaluate_base_token(&mut self) -> Option<Expr> {
        if let Some(token) = self.get_token() {
            match token {
                Token::Identifier(identifier) => Some(Expr::Identifier(identifier)),
                Token::String(str) => Some(Expr::String(str)),
                Token::Integer(int) => Some(Expr::Integer(int)),
                Token::Float(float) => Some(Expr::Float(float)),
                Token::Boolean(bool) => Some(Expr::Boolean(bool)),
                Token::Increment => Some(Expr::Unary {
                    op: UnaryOperators::Increment,
                    expr: {
                        self.advance();
                        let incr_token = self.evaluate_base_token();
                        if let Some(incr_value) = incr_token {
                            Some(Box::new(incr_value))
                        } else {
                            Some(Box::new(Expr::Integer(1)))
                        }
                    },
                }),
                Token::Decrement => Some(Expr::Unary {
                    op: UnaryOperators::Decrement,
                    expr: {
                        self.advance();
                        let decr_token = self.evaluate_base_token();
                        if let Some(decr_value) = decr_token {
                            Some(Box::new(decr_value))
                        } else {
                            Some(Box::new(Expr::Integer(1)))
                        }
                    },
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    fn evaluate_binary_operators(&mut self) -> Option<BinaryOperators> {
        if let Some(token) = self.get_token() {
            match token {
                Token::Plus => Some(BinaryOperators::Add),
                Token::Minus => Some(BinaryOperators::Sub),
                Token::Divider => Some(BinaryOperators::Div),
                Token::Multiplier => Some(BinaryOperators::Mul),
                Token::Modulo => Some(BinaryOperators::Mod),
                Token::Gt => Some(BinaryOperators::Gt),
                Token::Eq => Some(BinaryOperators::Eq),
                Token::NotEq => Some(BinaryOperators::NotEq),
                Token::GtEq => Some(BinaryOperators::GtEq),
                Token::Lt => Some(BinaryOperators::Lt),
                Token::LtEq => Some(BinaryOperators::LtEq),
                _ => None,
            }
        } else {
            None
        }
    }

    fn evaluate_assign(&mut self) -> Stmt {
        self.advance();
        let base_expr = self.evaluate_base_token();
        match base_expr {
            Some(val) => Stmt::Assign { value: val },
            None => panic!("Not found any assigned value"),
        }
    }
    fn evaluate_break(&mut self) -> Stmt {
        Stmt::ExprStmt(Expr::Break)
    }

    fn evaluate_local(&mut self) -> Stmt {
        return Stmt::Local {
            name: "local".to_string(),
        };
    }

    fn evaluate_integer(&mut self, int: i64) -> Stmt {
        let expr: Expr = Expr::Integer(int);
        return Stmt::ExprStmt(expr);
    }

    fn evaluate_float(&mut self, float: f64) -> Stmt {
        let expr = Expr::Float(float);
        return Stmt::ExprStmt(expr);
    }

    fn evaluate_string(&mut self, str: String) -> Stmt {
        let expr = Expr::String(str);
        Stmt::ExprStmt(expr)
    }

    fn evaluate_bool(&mut self, bool: bool) -> Stmt {
        let expr = Expr::Boolean(bool);
        Stmt::ExprStmt(expr)
    }

    fn evaluate_identifier(&mut self, identifier: String) -> Stmt {
        let expr = Expr::Identifier(identifier);
        Stmt::ExprStmt(expr)
    }

    fn evaluate_unary(&mut self) -> Stmt {
        Stmt::ExprStmt(Expr::Unary {
            op: UnaryOperators::Nil,
            expr: None,
        })
    }

    fn evaluate_for(&mut self) -> Stmt {
        self.advance();
        let identifier_token = self.get_token();

        let var = match identifier_token {
            Some(Token::Identifier(identifier)) => Expr::Identifier(identifier),
            _ => panic!("Expect Identifier in for"),
        };

        self.advance();

        let assign_token = self.get_token();
        let assign = match assign_token {
            Some(Token::Assign) => {
                self.advance();
                match self.evaluate_base_token() {
                    Some(expr) => Stmt::Assign { value: expr },
                    _ => panic!("Assign a initial value to for"),
                }
            }
            _ => panic!("Not found any assign"),
        };

        self.expect(Token::Comma);

        let end = self.evaluate_condition();

        self.expect(Token::Comma);

        self.advance(); // skip the Identifier
        let step = self.evaluate_base_token();

        self.expect(Token::Do);

        let body = self.evaluate_stmt();

        Stmt::ForNumeric {
            var,
            start: Box::new(assign),
            end: end,
            step: step,
            body: Box::new(body),
        }
    }

    fn expect(&mut self, expect_token: Token) {
        self.advance();
        if let Some(token) = self.get_token() {
            if token == expect_token {
                self.advance();
            } else {
                panic!("No expected token found: {:?}", expect_token)
            }
        } else {
            panic!("No token found as expected");
        }
    }
    fn advance(&mut self) {
        self.pos += 1;
    }

    fn next_peek(&mut self, nth: usize) -> Option<Token> {
        self.tokens.iter().nth(self.pos + nth).cloned()
    }
}
