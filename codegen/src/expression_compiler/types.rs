//! Core types and enums for the expression compiler
//!
//! This module defines the fundamental data types used throughout the
//! expression compilation pipeline.

use std::fmt;

/// Numeric type used in expressions
pub type Number = f64;

/// A compiled arithmetic expression that can generate Rust code
#[derive(Debug, Clone)]
pub struct CompiledExpression {
    pub original_expr: String,
    pub rpn_tokens: Vec<RpnToken>,
}

/// Token in Reverse Polish Notation
#[derive(Debug, Clone, PartialEq)]
pub enum RpnToken {
    Variable,           // Represents $val
    Number(Number),     // Numeric constant
    Operator(OpType),   // Arithmetic operator
    Function(FuncType), // Math function call
}

/// Arithmetic operator types
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpType {
    Add,
    Subtract, 
    Multiply,
    Divide,
}

/// Math function types
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FuncType {
    Int,  // int() - truncate to integer
    Exp,  // exp() - e^x
    Log,  // log() - natural logarithm
}

/// Internal token used during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseToken {
    Variable,
    Number(Number),
    Operator(Operator),
    Function(FuncType),
    LeftParen,
    RightParen,
}

/// Operator with precedence and associativity
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Operator {
    pub op_type: OpType,
    pub precedence: u8,
    pub is_left_associative: bool,
}

impl Operator {
    pub fn new(op_type: OpType, precedence: u8, is_left_associative: bool) -> Self {
        Self { op_type, precedence, is_left_associative }
    }
}

/// Helper trait for stack operations
pub trait Stack<T> {
    fn top(&self) -> Option<&T>;
}

impl<T> Stack<T> for Vec<T> {
    fn top(&self) -> Option<&T> {
        self.last()
    }
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Add => write!(f, "+"),
            OpType::Subtract => write!(f, "-"),
            OpType::Multiply => write!(f, "*"),
            OpType::Divide => write!(f, "/"),
        }
    }
}