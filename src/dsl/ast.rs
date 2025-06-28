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
  #[allow(dead_code)] Literal(String),
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
