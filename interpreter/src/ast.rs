use super::{expression::Expression, operator::Operator};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    StaticFunction(String, Vec<String>, Vec<AstNode>),
    Function(String, Vec<String>, Vec<AstNode>),
    Field(String, Expression),
    Operator(Operator, Vec<String>, Vec<AstNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstKind {
    ReturnStatement(Expression),
    Expression(Expression),
    IfStatement(Vec<(Expression, Vec<AstNode>)>),
    WhileStatement(Expression, Vec<AstNode>),
    VariableDefinition(String, Expression),
    ArrayVariableDefinition(Vec<String>, Expression),
    VariableAssignment(String, Expression),
    PlusAssignment(String, Expression),
    MinusAssignment(String, Expression),
    TimesAssignment(String, Expression),
    DivideAssignment(String, Expression),
    ClassDefinition(String, Vec<ClassMember>),
    FunctionCall(String, Vec<Expression>),
    ImportStatement(String),
    Comment(Vec<String>),
    Documentation(Vec<String>),
    BreakStatement,
    ContinueStatement,
    ExportStatement(Box<AstNode>),
    OperatorImplementation(Operator, Vec<String>, Vec<AstNode>),
    StaticFunctionDefinition(String, Vec<String>, Vec<AstNode>),
    FunctionDefinition(String, Vec<String>, Vec<AstNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    pub kind: AstKind,
    pub location: Range<usize>,
}

impl AstNode {
    pub fn new(kind: AstKind, location: Range<usize>) -> Self {
        AstNode { kind, location }
    }
}
