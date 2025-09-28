use std::fmt::Debug;

use crate::{lexer::Token, parser::Parser};

#[derive(Debug)]
pub struct Ast {
    program: String,
    body: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    If {
        condition: Expr,
        then_block: Box<Stmt>,
        else_block: Option<Box<Stmt>>,
        end: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Assign {
        value: Expr,
    },
    Local {
        name: String,
    },
    Print {
        value: Expr,
    },
    Block(Vec<Stmt>),
    ExprStmt(Expr),
    Return(Expr),
    Eof(),
}

#[derive(Debug)]
pub enum Expr {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Binary {
        op: BinaryOperators,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOperators,
        expr: Option<Box<Expr>>,
    },
    End,
}

#[derive(Debug)]
pub enum BinaryOperators {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOperators {
    Not,
    Nil,
}

impl Ast {
    pub fn new() -> Self {
        Self {
            program: String::from("Kernelino"),
            body: Vec::new(),
        }
    }

    pub fn build(&mut self, tokens: Vec<Token>) {
        let mut parser = Parser::new(tokens);
        parser.parse_tokens();
    }
}
