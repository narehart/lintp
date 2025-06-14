use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::dsl::ast::{BinaryOperator, Expression, StringTemplatePart, UnaryOperator};

/// Parser for the lintp DSL
/// This uses Pest to parse expressions according to the grammar defined in grammar.pest
#[derive(Parser)]
#[grammar = "dsl/grammar.pest"]
struct DslParser;

/// Parse a DSL expression string into an AST
///
/// # Arguments
/// * `input` - The expression string to parse
///
/// # Returns
/// * `Result<Expression>` - The parsed expression or an error
pub fn parse_expression(input: &str) -> Result<Expression> {
    // Parse the input using Pest - use top_level to enforce EOI
    let pairs = DslParser::parse(Rule::top_level, input)
        .map_err(|e| anyhow::anyhow!("Failed to parse expression: {}", e))?;

    // Get the first (and only) pair from the result
    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty parse result"))?;

    // Parse the expression from the pair
    parse_expression_pair(pair)
}

/// Parse a Pest pair into an Expression
fn parse_expression_pair(pair: pest::iterators::Pair<Rule>) -> Result<Expression> {
    match pair.as_rule() {
        Rule::top_level => {
            // The top_level rule contains or_expr ~ EOI
            // Just get the first inner rule which is or_expr
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow::anyhow!("Empty top-level expression"))?;
            parse_expression_pair(inner)
        }

        Rule::expression => {
            // The expression rule contains just or_expr
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow::anyhow!("Empty expression"))?;
            parse_expression_pair(inner)
        }

        Rule::or_expr => parse_binary_expr(pair, Rule::or_op, BinaryOperator::Or),
        Rule::and_expr => parse_binary_expr(pair, Rule::and_op, BinaryOperator::And),

        Rule::not_expr => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();

            if first.as_rule() == Rule::not_op {
                let expr = parse_expression_pair(inner.next().unwrap())?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Not,
                    expr: Box::new(expr),
                })
            } else {
                parse_expression_pair(first)
            }
        }

        Rule::comparison_expr => {
            let mut inner = pair.into_inner();
            let left = parse_expression_pair(inner.next().unwrap())?;

            if let Some(op_pair) = inner.next() {
                let op = match op_pair.as_str() {
                    "==" => BinaryOperator::Equal,
                    "!=" => BinaryOperator::NotEqual,
                    "<" => BinaryOperator::LessThan,
                    ">" => BinaryOperator::GreaterThan,
                    "<=" => BinaryOperator::LessThanOrEqual,
                    ">=" => BinaryOperator::GreaterThanOrEqual,
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Unknown comparison operator: {}",
                            op_pair.as_str()
                        ));
                    }
                };

                let right = parse_expression_pair(inner.next().unwrap())?;

                Ok(Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            } else {
                Ok(left)
            }
        }

        Rule::add_expr | Rule::mul_expr => {
            // For simplicity, we're not implementing arithmetic operators in this example
            // but we parse them as if they were there
            let mut inner = pair.into_inner();
            parse_expression_pair(inner.next().unwrap())
        }

        Rule::unary_expr => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();

            if first.as_rule() == Rule::minus_op {
                let expr = parse_expression_pair(inner.next().unwrap())?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Minus,
                    expr: Box::new(expr),
                })
            } else {
                parse_expression_pair(first)
            }
        }

        Rule::primary => {
            let inner = pair.into_inner().next().unwrap();
            parse_expression_pair(inner)
        }

        Rule::variable => {
            let name = pair.as_str()[1..].to_string(); // Remove the $ prefix
            Ok(Expression::Variable(name))
        }

        Rule::string_literal => {
            let s = pair.as_str();
            let is_double_quoted = s.starts_with('"');
            let content = &s[1..s.len() - 1]; // Remove quotes

            if is_double_quoted && content.contains("${") {
                // Double-quoted string with possible templates
                let mut template_parts = Vec::new();
                let mut current_pos = 0;

                while current_pos < content.len() {
                    if let Some(start) = content[current_pos..].find("${") {
                        let abs_start = current_pos + start;

                        // Add any literal text before the template
                        if start > 0 {
                            let literal_text = &content[current_pos..abs_start];
                            if !literal_text.is_empty() {
                                template_parts
                                    .push(StringTemplatePart::Literal(literal_text.to_string()));
                            }
                        }

                        // Find the matching closing brace
                        let mut brace_count = 1;
                        let mut end_pos = abs_start + 2; // Start after "${"

                        while end_pos < content.len() && brace_count > 0 {
                            match content.chars().nth(end_pos).unwrap() {
                                '{' => brace_count += 1,
                                '}' => brace_count -= 1,
                                _ => {}
                            }
                            end_pos += 1;
                        }

                        if brace_count == 0 {
                            // Found matching brace
                            let expr_content = &content[abs_start + 2..end_pos - 1];

                            // Parse the expression
                            let inner_pairs = DslParser::parse(Rule::expression, expr_content)
                                .map_err(|e| {
                                    anyhow::anyhow!("Failed to parse template expression: {}", e)
                                })?;

                            let inner_pair = inner_pairs
                                .into_iter()
                                .next()
                                .ok_or_else(|| anyhow::anyhow!("Empty template expression"))?;

                            let inner_expr = parse_expression_pair(inner_pair)?;
                            template_parts
                                .push(StringTemplatePart::Expression(Box::new(inner_expr)));

                            current_pos = end_pos;
                        } else {
                            // Unmatched braces, treat as literal
                            template_parts.push(StringTemplatePart::Literal(
                                content[current_pos..].to_string(),
                            ));
                            break;
                        }
                    } else {
                        // No more templates, add remaining text
                        let remaining = &content[current_pos..];
                        if !remaining.is_empty() {
                            template_parts.push(StringTemplatePart::Literal(remaining.to_string()));
                        }
                        break;
                    }
                }

                if template_parts
                    .iter()
                    .any(|p| matches!(p, StringTemplatePart::Expression(_)))
                {
                    Ok(Expression::StringTemplate(template_parts))
                } else {
                    // No actual templates found, return as string literal
                    Ok(Expression::StringLiteral(content.to_string()))
                }
            } else {
                // Single-quoted string or double-quoted without templates
                // Process escape sequences
                let mut processed = String::new();
                let mut chars = content.chars().peekable();

                while let Some(c) = chars.next() {
                    if c == '\\' && chars.peek().is_some() {
                        let next = chars.next().unwrap();
                        match next {
                            'n' => processed.push('\n'),
                            'r' => processed.push('\r'),
                            't' => processed.push('\t'),
                            '\\' => processed.push('\\'),
                            '\'' => processed.push('\''),
                            '"' => processed.push('"'),
                            _ => processed.push(next),
                        }
                    } else {
                        processed.push(c);
                    }
                }

                Ok(Expression::StringLiteral(processed))
            }
        }

        Rule::integer_literal => {
            let value = pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| anyhow::anyhow!("Failed to parse integer: {}", e))?;
            Ok(Expression::IntegerLiteral(value))
        }

        Rule::boolean_literal => {
            let value = match pair.as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid boolean literal: {}",
                        pair.as_str()
                    ));
                }
            };
            Ok(Expression::BooleanLiteral(value))
        }

        Rule::regex_literal => {
            let s = pair.as_str();

            // Make sure it's a valid regex pattern
            if s.len() < 3 || !s.starts_with('/') || !s.ends_with('/') {
                return Err(anyhow::anyhow!("Invalid regex literal: {}", s));
            }

            let pattern = &s[1..s.len() - 1]; // Remove the / delimiters

            // Process escape sequences in the regex
            let mut processed = String::new();
            let mut chars = pattern.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '\\' && chars.peek().is_some() {
                    // Handle escape sequence
                    let next = chars.next().unwrap();
                    processed.push('\\');
                    processed.push(next);
                } else {
                    processed.push(c);
                }
            }

            Ok(Expression::RegexLiteral(processed))
        }

        Rule::list_literal => {
            let mut items = Vec::new();

            for item_pair in pair.into_inner() {
                if item_pair.as_rule() == Rule::expression {
                    let item = parse_expression_pair(item_pair)?;
                    items.push(item);
                }
            }

            Ok(Expression::ListLiteral(items))
        }

        Rule::string_template => {
            // Extract the expression inside ${...}
            let template_str = pair.as_str();
            if template_str.len() < 4
                || !template_str.starts_with("${")
                || !template_str.ends_with("}")
            {
                return Err(anyhow::anyhow!("Invalid string template: {}", template_str));
            }

            let inner_expr_str = &template_str[2..template_str.len() - 1];

            // Parse the inner expression
            let inner_pairs = DslParser::parse(Rule::expression, inner_expr_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse template expression: {}", e))?;

            let inner_pair = inner_pairs
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("Empty template expression"))?;

            let inner_expr = parse_expression_pair(inner_pair)?;

            // Create a template with just one expression part
            let parts = vec![StringTemplatePart::Expression(Box::new(inner_expr))];
            Ok(Expression::StringTemplate(parts))
        }

        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name_pair = inner.next().unwrap();

            if name_pair.as_rule() != Rule::identifier {
                return Err(anyhow::anyhow!("Expected identifier for function name"));
            }

            let name = name_pair.as_str().to_string();
            let mut args = Vec::new();

            for arg_pair in inner {
                if arg_pair.as_rule() == Rule::expression {
                    let arg = parse_expression_pair(arg_pair)?;
                    args.push(arg);
                }
            }

            Ok(Expression::FunctionCall { name, args })
        }

        Rule::reference => {
            let name = pair.as_str().to_string();
            Ok(Expression::Reference(name))
        }

        Rule::index_expr => {
            let mut inner = pair.into_inner();
            let base_pair = inner.next().unwrap();
            let mut expr = parse_expression_pair(base_pair)?;

            for chunk in inner {
                if chunk.as_rule() == Rule::expression {
                    let index = parse_expression_pair(chunk)?;
                    expr = Expression::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    };
                }
            }

            Ok(expr)
        }

        Rule::base_expr => {
            let inner = pair.into_inner().next().unwrap();
            parse_expression_pair(inner)
        }

        _ => Err(anyhow::anyhow!("Unexpected rule: {:?}", pair.as_rule())),
    }
}

/// Parse a binary expression (AND/OR)
fn parse_binary_expr(
    pair: pest::iterators::Pair<Rule>,
    op_rule: Rule,
    op_kind: BinaryOperator,
) -> Result<Expression> {
    let mut inner = pair.into_inner();
    let mut left = parse_expression_pair(inner.next().unwrap())?;

    // Process all operators of the same kind
    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == op_rule {
            let right = parse_expression_pair(inner.next().unwrap())?;

            left = Expression::BinaryOp {
                op: op_kind.clone(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_expressions() -> Result<()> {
        // Test variable
        let expr = parse_expression("$NAME")?;
        assert!(matches!(expr, Expression::Variable(name) if name == "NAME"));

        // Test string literal
        let expr = parse_expression("\"hello world\"")?;
        assert!(matches!(expr, Expression::StringLiteral(s) if s == "hello world"));

        // Test boolean literal
        let expr = parse_expression("true")?;
        assert!(matches!(expr, Expression::BooleanLiteral(true)));

        // Test regex literal
        let expr = parse_expression("/^test-[0-9]+$/")?;
        assert!(matches!(expr, Expression::RegexLiteral(pattern) if pattern == "^test-[0-9]+$"));

        Ok(())
    }

    #[test]
    fn test_logical_operators() -> Result<()> {
        // Test AND
        let expr = parse_expression("$NAME == \"test\" && $EXT == \"js\"")?;

        if let Expression::BinaryOp { op, .. } = expr {
            assert_eq!(op, BinaryOperator::And);
        } else {
            panic!("Expected BinaryOp");
        }

        // Test OR
        let expr = parse_expression("matches($NAME, /test/) || in($NAME, [\"foo\", \"bar\"])")?;

        if let Expression::BinaryOp { op, .. } = expr {
            assert_eq!(op, BinaryOperator::Or);
        } else {
            panic!("Expected BinaryOp");
        }

        // Test NOT
        let expr = parse_expression("!matches($NAME, /test/)")?;

        if let Expression::UnaryOp { op, .. } = expr {
            assert_eq!(op, UnaryOperator::Not);
        } else {
            panic!("Expected UnaryOp");
        }

        Ok(())
    }

    #[test]
    fn test_function_calls() -> Result<()> {
        // Test simple function
        let expr = parse_expression("matches($NAME, /test/)")?;

        if let Expression::FunctionCall { name, args } = expr {
            assert_eq!(name, "matches");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected FunctionCall");
        }

        // Test nested function calls
        let expr =
            parse_expression("any(map(siblings(\"*.js\"), without($item, \".js\")), $NAME)")?;

        if let Expression::FunctionCall { name, .. } = expr {
            assert_eq!(name, "any");
        } else {
            panic!("Expected FunctionCall");
        }

        Ok(())
    }

    #[test]
    fn test_complex_expressions() -> Result<()> {
        // Test complex expression
        let expr = parse_expression(
      "matches($NAME, /^test-[0-9]+$/) || (in($EXT, [\"js\", \"ts\"]) && !exists(\"*.tmp\"))"
    )?;

        if let Expression::BinaryOp { op, .. } = expr {
            assert_eq!(op, BinaryOperator::Or);
        } else {
            panic!("Expected BinaryOp");
        }

        // Test string template
        let expr = parse_expression("matches($NAME, ${siblings(\"*.js\")[0]})")?;

        if let Expression::FunctionCall { name, args } = expr {
            assert_eq!(name, "matches");
            assert_eq!(args.len(), 2);

            if let Expression::StringTemplate(_) = &args[1] {
                // Expected
            } else {
                panic!("Expected StringTemplate");
            }
        } else {
            panic!("Expected FunctionCall");
        }

        Ok(())
    }
}
