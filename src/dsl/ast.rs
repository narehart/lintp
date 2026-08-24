//! The parsed form of a rule expression.
//!
//! A rule like `kebab-case && $EXT == "ts"` becomes a tree of
//! [`Expression`](crate::dsl::ast::Expression)s, which
//! [`crate::dsl::evaluator`] walks against one file at a time. The
//! [`std::fmt::Display`] impls render the tree back to DSL source so failure
//! messages can quote the exact subexpression that failed.

/// A node in a parsed rule expression.
#[derive(Debug, Clone)]
pub enum Expression {
    /// A built-in variable reference: `$NAME`, `$EXT`, `$item`.
    Variable(String),

    /// A quoted string with no interpolation: `"test"`.
    StringLiteral(String),
    /// A whole number, used by `count()` comparisons and `exists()` bounds.
    IntegerLiteral(i64),
    /// `true` or `false`, and the whole rule when a key is a constant.
    BooleanLiteral(bool),
    /// A `/pattern/` regex, passed to `matches()`.
    RegexLiteral(String),
    /// A bracketed list: `["a", "b"]`, the right-hand side of `in()`.
    ListLiteral(Vec<Expression>),

    /// Subscript into a list value: `siblings("*")[0]`.
    Index {
        /// The list being indexed.
        expr: Box<Expression>,
        /// The position to read.
        index: Box<Expression>,
    },

    /// Two operands joined by an operator: `$EXT == "ts"`.
    BinaryOp {
        /// Which operator joins the operands.
        op: BinaryOperator,
        /// Left-hand operand.
        left: Box<Expression>,
        /// Right-hand operand.
        right: Box<Expression>,
    },

    /// A prefix operator applied to one operand: `!kebab-case`.
    UnaryOp {
        /// Which operator is applied.
        op: UnaryOperator,
        /// The operand it applies to.
        expr: Box<Expression>,
    },

    /// A built-in call: `matches($BASENAME, /^[a-z]+$/)`.
    FunctionCall {
        /// The function's name as written in the rule.
        name: String,
        /// Its arguments, in source order.
        args: Vec<Expression>,
    },

    /// A bare name resolving to an entry under `custom-matchers`.
    Reference(String),

    /// A string with `${...}` interpolation: `"${$BASENAME}.test.ts"`.
    StringTemplate(Vec<StringTemplatePart>),
}

/// One segment of an interpolated string.
#[derive(Debug, Clone)]
pub enum StringTemplatePart {
    /// Text carried through as written.
    Literal(String),
    /// A `${...}` hole, evaluated and stringified at match time.
    Expression(Box<Expression>),
}

/// An operator taking a left and a right operand.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    /// `&&` — short-circuits, and drives the `(failed: ...)` breakdown.
    And,
    /// `||` — short-circuits.
    Or,
    /// `==`
    Equal,
    /// `!=`
    NotEqual,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `<=`
    LessThanOrEqual,
    /// `>=`
    GreaterThanOrEqual,
}

/// An operator taking a single operand.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    /// `!` — boolean negation.
    Not,
    /// `-` — numeric negation.
    Minus,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::LessThanOrEqual => "<=",
            BinaryOperator::GreaterThanOrEqual => ">=",
        };
        write!(f, "{s}")
    }
}

/// Renders expressions back to DSL source form, primarily so failure
/// messages can point at the specific subexpression that failed.
impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Variable(name) => write!(f, "${name}"),
            Expression::StringLiteral(s) => write!(f, "\"{s}\""),
            Expression::IntegerLiteral(i) => write!(f, "{i}"),
            Expression::BooleanLiteral(b) => write!(f, "{b}"),
            Expression::RegexLiteral(pattern) => write!(f, "/{pattern}/"),
            Expression::ListLiteral(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Expression::Index { expr, index } => write!(f, "{expr}[{index}]"),
            Expression::BinaryOp { op, left, right } => {
                // Parenthesize nested boolean operators for readability
                let needs_parens = |e: &Expression| {
                    matches!(
                        e,
                        Expression::BinaryOp {
                            op: BinaryOperator::And | BinaryOperator::Or,
                            ..
                        }
                    )
                };
                if needs_parens(left) {
                    write!(f, "({left})")?;
                } else {
                    write!(f, "{left}")?;
                }
                write!(f, " {op} ")?;
                if needs_parens(right) {
                    write!(f, "({right})")
                } else {
                    write!(f, "{right}")
                }
            }
            Expression::UnaryOp { op, expr } => match op {
                UnaryOperator::Not => write!(f, "!{expr}"),
                UnaryOperator::Minus => write!(f, "-{expr}"),
            },
            Expression::FunctionCall { name, args } => {
                write!(f, "{name}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Expression::Reference(name) => write!(f, "{name}"),
            Expression::StringTemplate(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        StringTemplatePart::Literal(s) => write!(f, "{s}")?,
                        StringTemplatePart::Expression(expr) => write!(f, "${{{expr}}}")?,
                    }
                }
                write!(f, "\"")
            }
        }
    }
}
