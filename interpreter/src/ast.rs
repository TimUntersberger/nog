use super::{expression::Expression, operator::Operator};

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    StaticFunction(String, Vec<String>, Vec<Ast>),
    Function(String, Vec<String>, Vec<Ast>),
    Field(String, Expression),
    Operator(Operator, Vec<String>, Vec<Ast>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    ReturnStatement(Expression),
    Expression(Expression),
    IfStatement(Vec<(Expression, Vec<Ast>)>),
    WhileStatement(Expression, Vec<Ast>),
    VariableDefinition(String, Expression),
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
    ExportStatement(Box<Ast>),
    OperatorImplementation(Operator, Vec<String>, Vec<Ast>),
    StaticFunctionDefinition(String, Vec<String>, Vec<Ast>),
    FunctionDefinition(String, Vec<String>, Vec<Ast>),
}
