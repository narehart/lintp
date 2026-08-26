//! Tests for rendering a parsed expression back to DSL source.
//!
//! This is what failure messages quote, so a wrong rendering sends someone
//! looking at the wrong part of their rule. Every expression form and every
//! operator has to survive the round trip.

use anyhow::Result;

use lintp::dsl::parser::parse_expression;

/// Parse `src` and render it back out.
fn render(src: &str) -> Result<String> {
    Ok(parse_expression(src)?.to_string())
}

#[test]
fn renders_every_binary_operator() -> Result<()> {
    for (src, expected) in [
        ("$A && $B", "$A && $B"),
        ("$A || $B", "$A || $B"),
        ("$A == $B", "$A == $B"),
        ("$A != $B", "$A != $B"),
        ("$A < $B", "$A < $B"),
        ("$A > $B", "$A > $B"),
        ("$A <= $B", "$A <= $B"),
        ("$A >= $B", "$A >= $B"),
    ] {
        assert_eq!(render(src)?, expected, "rendering {src}");
    }

    Ok(())
}

#[test]
fn renders_unary_operators() -> Result<()> {
    assert_eq!(render("!$NAME")?, "!$NAME");
    assert_eq!(render("-1")?, "-1");

    Ok(())
}

#[test]
fn renders_literals_in_their_source_form() -> Result<()> {
    assert_eq!(render("\"test\"")?, "\"test\"");
    assert_eq!(render("42")?, "42");
    assert_eq!(render("true")?, "true");
    assert_eq!(render("false")?, "false");
    assert_eq!(render("/^[a-z]+$/")?, "/^[a-z]+$/");

    Ok(())
}

#[test]
fn renders_lists_comma_separated() -> Result<()> {
    assert_eq!(render("[\"a\", \"b\", \"c\"]")?, "[\"a\", \"b\", \"c\"]");
    assert_eq!(render("[]")?, "[]");

    Ok(())
}

#[test]
fn renders_function_calls_with_their_arguments() -> Result<()> {
    assert_eq!(render("count($NAME)")?, "count($NAME)");
    assert_eq!(
        render("matches($BASENAME, /^[a-z]+$/)")?,
        "matches($BASENAME, /^[a-z]+$/)"
    );
    assert_eq!(render("exists(\"a\", 1, 2)")?, "exists(\"a\", 1, 2)");

    Ok(())
}

#[test]
fn renders_a_matcher_reference_as_a_bare_name() -> Result<()> {
    assert_eq!(render("kebab-case")?, "kebab-case");

    Ok(())
}

#[test]
fn renders_indexing() -> Result<()> {
    assert_eq!(render("siblings(\"*\")[0]")?, "siblings(\"*\")[0]");

    Ok(())
}

#[test]
fn renders_string_templates_with_their_holes() -> Result<()> {
    assert_eq!(
        render("\"${$BASENAME}.test.ts\"")?,
        "\"${$BASENAME}.test.ts\""
    );

    Ok(())
}

/// Nested boolean operators are parenthesised so the rendering cannot be
/// read with the wrong precedence.
#[test]
fn parenthesises_nested_boolean_operators() -> Result<()> {
    assert_eq!(render("$A && ($B || $C)")?, "$A && ($B || $C)");
    assert_eq!(render("($A || $B) && $C")?, "($A || $B) && $C");

    Ok(())
}

/// Comparisons are not parenthesised: they cannot be misread the way a
/// nested && / || chain can.
#[test]
fn leaves_comparisons_unparenthesised() -> Result<()> {
    assert_eq!(render("$EXT == \"ts\" && $A")?, "$EXT == \"ts\" && $A");

    Ok(())
}
