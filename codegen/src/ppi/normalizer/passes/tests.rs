//! Tests for normalization passes

use super::*;
use crate::ppi::normalizer::NormalizationPass;
use crate::ppi::types::PpiNode;

#[test]
fn test_safe_division_normalization() {
    // Create AST for: $val ? 1 / $val : 0
    let ast = PpiNode {
        class: "PPI::Statement".to_string(),
        content: None,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
        children: vec![
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("?".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Number".to_string(),
                content: Some("1".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: Some(1.0),
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("/".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some(":".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Number".to_string(),
                content: Some("0".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: Some(0.0),
                string_value: None,
                structure_bounds: None,
            },
        ],
    };

    let normalizer = SafeDivisionNormalizer;
    let result = normalizer.transform(ast);

    assert_eq!(result.class, "FunctionCall");
    assert_eq!(result.content.as_deref(), Some("safe_reciprocal"));
    assert_eq!(result.children.len(), 1);
}

#[test]
fn test_function_call_normalization() {
    // Create AST for: length $val
    let ast = PpiNode {
        class: "PPI::Statement".to_string(),
        content: None,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
        children: vec![
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("length".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
    };

    let normalizer = FunctionCallNormalizer;
    let result = normalizer.transform(ast);

    assert_eq!(result.class, "FunctionCall");
    assert_eq!(result.content.as_deref(), Some("length"));
    assert_eq!(result.children.len(), 1);
}

#[test]
fn test_string_concat_normalization() {
    // Create AST for: "a" . "b"
    let ast = PpiNode {
        class: "PPI::Statement".to_string(),
        content: None,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
        children: vec![
            PpiNode {
                class: "PPI::Token::Quote::Double".to_string(),
                content: Some("\"a\"".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: Some("a".to_string()),
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some(".".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Quote::Double".to_string(),
                content: Some("\"b\"".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: Some("b".to_string()),
                structure_bounds: None,
            },
        ],
    };

    let normalizer = StringOpNormalizer;
    let result = normalizer.transform(ast);

    assert_eq!(result.class, "StringConcat");
    assert_eq!(result.children.len(), 2);
}

#[test]
fn test_postfix_return_normalization() {
    // Create AST for: return "n/a" if $val =~ /undef/;
    let ast = PpiNode {
        class: "PPI::Statement::Break".to_string(),
        content: None,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
        children: vec![
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("return".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Quote::Double".to_string(),
                content: Some("\"n/a\"".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: Some("n/a".to_string()),
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("if".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("=~".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Regexp::Match".to_string(),
                content: Some("/undef/".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Structure".to_string(),
                content: Some(";".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
    };

    let normalizer = PostfixConditionalNormalizer;
    let result = normalizer.transform(ast);

    // Should be transformed to IfStatement
    assert_eq!(result.class, "IfStatement");
    assert_eq!(result.children.len(), 2);

    // First child should be the condition
    assert_eq!(result.children[0].class, "PPI::Statement::Expression");

    // Second child should be the block with return statement
    assert_eq!(result.children[1].class, "PPI::Structure::Block");
    let block_stmt = &result.children[1].children[0];
    assert_eq!(block_stmt.class, "PPI::Statement");

    // Block should contain return keyword + expression + semicolon
    assert_eq!(block_stmt.children.len(), 3);
    assert_eq!(block_stmt.children[0].content, Some("return".to_string()));
    assert_eq!(block_stmt.children[1].content, Some("\"n/a\"".to_string()));
    assert_eq!(block_stmt.children[2].content, Some(";".to_string()));
}

#[test]
fn test_postfix_assignment_normalization() {
    // Create AST for: $val = 42 if $condition;
    let ast = PpiNode {
        class: "PPI::Statement".to_string(),
        content: None,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
        children: vec![
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("=".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Number".to_string(),
                content: Some("42".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: Some(42.0),
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("if".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$condition".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Structure".to_string(),
                content: Some(";".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ],
    };

    let normalizer = PostfixConditionalNormalizer;
    let result = normalizer.transform(ast);

    // Should be transformed to IfStatement
    assert_eq!(result.class, "IfStatement");
    assert_eq!(result.children.len(), 2);

    // First child should be the condition
    assert_eq!(result.children[0].class, "PPI::Statement::Expression");
    assert_eq!(result.children[0].children.len(), 1);
    assert_eq!(
        result.children[0].children[0].content,
        Some("$condition".to_string())
    );

    // Second child should be the block with assignment
    assert_eq!(result.children[1].class, "PPI::Structure::Block");
    let block_stmt = &result.children[1].children[0];
    assert_eq!(block_stmt.class, "PPI::Statement");

    // Block should contain assignment: $val = 42
    assert_eq!(block_stmt.children.len(), 3);
    assert_eq!(block_stmt.children[0].content, Some("$val".to_string()));
    assert_eq!(block_stmt.children[1].content, Some("=".to_string()));
    assert_eq!(block_stmt.children[2].content, Some("42".to_string()));
}
