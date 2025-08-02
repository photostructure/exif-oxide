//! Core types and enums for the expression compiler
//!
//! This module defines the fundamental data types used throughout the
//! expression compilation pipeline, using AST-based approach for ternary support.

use std::fmt;

/// Numeric type used in expressions
pub type Number = f64;

/// A compiled expression that can generate Rust code using AST representation
#[derive(Debug, Clone)]
pub struct CompiledExpression {
    pub original_expr: String,
    pub ast: Box<AstNode>,
}

/// Abstract Syntax Tree node for expression representation
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Variable reference ($val)
    Variable,
    /// Numeric literal
    Number(Number),
    /// String literal with optional variable interpolation
    String { value: String, has_interpolation: bool },
    /// ExifTool's undef value
    Undefined,
    /// Binary arithmetic operation
    BinaryOp { op: OpType, left: Box<AstNode>, right: Box<AstNode> },
    /// Comparison operation  
    ComparisonOp { op: CompType, left: Box<AstNode>, right: Box<AstNode> },
    /// Ternary conditional expression
    TernaryOp { condition: Box<AstNode>, true_expr: Box<AstNode>, false_expr: Box<AstNode> },
    /// Function call
    FunctionCall { func: FuncType, arg: Box<AstNode> },
    /// ExifTool function call (Image::ExifTool::Module::Function)
    ExifToolFunction { name: String, arg: Box<AstNode> },
    /// Sprintf function call with format string and arguments
    Sprintf { format_string: String, args: Vec<Box<AstNode>> },
    /// Unary minus operation
    UnaryMinus { operand: Box<AstNode> },
}


/// Arithmetic operator types
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpType {
    Add,
    Subtract, 
    Multiply,
    Divide,
    Power,       // Power operation (Perl's ** operator)
    Concatenate, // String concatenation (Perl's . operator)
}

/// Comparison operator types for ternary conditions
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompType {
    GreaterEq,    // >=
    Greater,      // >
    LessEq,       // <=
    Less,         // <
    Equal,        // ==
    NotEqual,     // !=
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
    String(String),
    Undefined,
    Operator(Operator),
    Comparison(CompOperator),
    Function(FuncType),
    LeftParen,
    RightParen,
    Question,     // ?
    Colon,        // :
    Sprintf,      // sprintf function call
    ExifToolFunction(String), // Image::ExifTool::Module::Function
    Comma,        // , (internal parsing token, not exposed in AST)
    UnaryMinus,   // - (unary minus operator)
}

/// Comparison operator with precedence
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CompOperator {
    pub comp_type: CompType,
    pub precedence: u8,
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

impl CompOperator {
    pub fn new(comp_type: CompType, precedence: u8) -> Self {
        Self { comp_type, precedence }
    }
}


impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Add => write!(f, "+"),
            OpType::Subtract => write!(f, "-"),
            OpType::Multiply => write!(f, "*"),
            OpType::Divide => write!(f, "/"),
            OpType::Power => write!(f, "**"),
            OpType::Concatenate => write!(f, "."),
        }
    }
}

impl fmt::Display for CompType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompType::GreaterEq => write!(f, ">="),
            CompType::Greater => write!(f, ">"),
            CompType::LessEq => write!(f, "<="),
            CompType::Less => write!(f, "<"),
            CompType::Equal => write!(f, "=="),
            CompType::NotEqual => write!(f, "!="),
        }
    }
}