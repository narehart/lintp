#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String),

    StringLiteral(String),
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    RegexLiteral(String),
    ListLiteral(Vec<Expression>),

    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },

    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },

    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    Reference(String), // Reference to a custom matcher

    StringTemplate(Vec<StringTemplatePart>),
}

#[derive(Debug, Clone)]
pub enum StringTemplatePart {
    #[allow(dead_code)]
    Literal(String),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
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
        write!(f, "{}", s)
    }
}

/// Renders expressions back to DSL source form, primarily so failure
/// messages can point at the specific subexpression that failed.
impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Variable(name) => write!(f, "${}", name),
            Expression::StringLiteral(s) => write!(f, "\"{}\"", s),
            Expression::IntegerLiteral(i) => write!(f, "{}", i),
            Expression::BooleanLiteral(b) => write!(f, "{}", b),
            Expression::RegexLiteral(pattern) => write!(f, "/{}/", pattern),
            Expression::ListLiteral(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Expression::Index { expr, index } => write!(f, "{}[{}]", expr, index),
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
                    write!(f, "({})", left)?;
                } else {
                    write!(f, "{}", left)?;
                }
                write!(f, " {} ", op)?;
                if needs_parens(right) {
                    write!(f, "({})", right)
                } else {
                    write!(f, "{}", right)
                }
            }
            Expression::UnaryOp { op, expr } => match op {
                UnaryOperator::Not => write!(f, "!{}", expr),
                UnaryOperator::Minus => write!(f, "-{}", expr),
            },
            Expression::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expression::Reference(name) => write!(f, "{}", name),
            Expression::StringTemplate(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        StringTemplatePart::Literal(s) => write!(f, "{}", s)?,
                        StringTemplatePart::Expression(expr) => write!(f, "${{{}}}", expr)?,
                    }
                }
                write!(f, "\"")
            }
        }
    }
}
