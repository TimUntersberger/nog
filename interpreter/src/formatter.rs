use std::fmt::Display;

use itertools::Itertools;

use super::{
    ast::ClassMember,
    ast::{AstKind, AstNode},
    expression::{Expression, ExpressionKind},
    interpreter::Program,
    operator::Operator,
};

pub struct Formatter<'a> {
    prog: &'a Program<'a>,
    level: usize,
}

impl<'a> Formatter<'a> {
    pub fn new(prog: &'a Program) -> Self {
        Self { prog, level: 0 }
    }

    pub fn format_expr(&mut self, expr: &Expression) -> String {
        match &expr.kind {
            ExpressionKind::Null => "null".into(),
            ExpressionKind::StringLiteral(text) => format!(
                "\"{}\"",
                text.replace("\r", "\\r")
                    .replace("\n", "\\n")
                    .replace("\t", "\\t")
                    .replace("\"", "\\\"")
            ),
            ExpressionKind::Identifier(text)
            | ExpressionKind::ClassIdentifier(text)
            | ExpressionKind::NumberLiteral(text)
            | ExpressionKind::HexLiteral(text)
            | ExpressionKind::BooleanLiteral(text) => text.clone(),
            ExpressionKind::ArrayLiteral(items) => format!(
                "[{}]",
                items
                    .into_iter()
                    .map(|expr| self.format_expr(expr))
                    .join(", ")
            ),
            ExpressionKind::ObjectLiteral(fields) => {
                if fields.is_empty() {
                    format!("#{{}}")
                } else {
                    format!(
                        "#{{\n{}\n{}}}",
                        {
                            self.level += 1;
                            let body = fields
                                .iter()
                                .map(|(k, v)| {
                                    format!(
                                        "{}\"{}\": {}",
                                        self.indentation(),
                                        k,
                                        self.format_expr(v)
                                    )
                                })
                                .join(",\n");
                            self.level -= 1;
                            body
                        },
                        self.indentation()
                    )
                }
            }
            ExpressionKind::ArrowFunction(args, body) => {
                if body.len() == 1 {
                    let body = body.get(0).unwrap();
                    match &body.kind {
                        AstKind::ReturnStatement(expr) | AstKind::Expression(expr) => {
                            let body = self.format_expr(&expr);
                            format!(
                                "{} => {}",
                                if args.len() == 1 {
                                    args[0].clone()
                                } else {
                                    format!("({})", self.format_args(args))
                                },
                                body
                            )
                        }
                        actual => unreachable!("{:?}", actual),
                    }
                } else {
                    self.level += 1;
                    let body = self.format_stmts(body);
                    self.level -= 1;

                    format!(
                        "{} => {{\n{}\n{}}}",
                        if args.len() == 1 {
                            args[0].clone()
                        } else {
                            format!("({})", self.format_args(args))
                        },
                        body,
                        self.indentation()
                    )
                }
            }
            ExpressionKind::ClassInstantiation(name, fields) => format!(
                "{}{{{}}}",
                name,
                fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_expr(v)))
                    .join("\n")
            ),
            ExpressionKind::PreOp(op, expr) => {
                format!("{}{}", op.to_string(), self.format_expr(expr))
            }
            ExpressionKind::BinaryOp(lhs, op, rhs) => match op {
                Operator::Dot => format!(
                    "{}{}{}",
                    self.format_expr(lhs),
                    op.to_string(),
                    self.format_expr(rhs)
                ),
                _ => format!(
                    "{} {} {}",
                    self.format_expr(lhs),
                    op.to_string(),
                    self.format_expr(rhs)
                ),
            },
            ExpressionKind::PostOp(lhs, op, value) => match op {
                Operator::Call => format!("{}({})", self.format_expr(lhs), {
                    if let ExpressionKind::ArrayLiteral(exprs) = &value.as_ref().unwrap().kind {
                        exprs.iter().map(|expr| self.format_expr(expr)).join(", ")
                    } else {
                        self.format_expr(value.as_ref().unwrap())
                    }
                }),
                Operator::Index => format!(
                    "{}[{}]",
                    lhs.to_string(),
                    value.as_ref().unwrap().to_string()
                ),
                _ => format!("{}{}", lhs.to_string(), op.to_string()),
            },
        }
    }

    fn format_args<T: Display>(&mut self, args: &Vec<T>) -> String {
        args.iter().map(|a| a.to_string()).join(", ")
    }

    fn get_line_ending(&self, ast: &AstNode) -> String {
        match ast {
            // Ast::FunctionDefinition(_, _, _) => "",
            _ => "",
        }
        .into()
    }

    fn indentation(&self) -> String {
        "    ".repeat(self.level)
    }

    fn format_ast(&mut self, ast: &AstNode) -> String {
        match &ast.kind {
            AstKind::VariableDefinition(name, value) => {
                format!("var {} = {}", name, self.format_expr(&value))
            }
            AstKind::ArrayVariableDefinition(names, value) => {
                format!("var [{}] = {}", names.join(", "), self.format_expr(&value))
            }
            AstKind::VariableAssignment(name, value) => {
                format!("{} = {}", name, self.format_expr(&value))
            }
            AstKind::Documentation(lines) => lines
                .iter()
                .map(|line| format!("///{}", line))
                .join(&format!("\n{}", self.indentation())),
            AstKind::Comment(lines) => lines
                .iter()
                .map(|line| format!("//{}", line))
                .join(&format!("\n{}", self.indentation())),
            AstKind::BreakStatement => "break".into(),
            AstKind::ReturnStatement(expr) => format!("return {}", self.format_expr(&expr)),
            AstKind::ExportStatement(stmt) => format!("export {}", self.format_ast(&stmt)),
            AstKind::ImportStatement(path) => format!("import {}", path),
            AstKind::IfStatement(branches) => branches
                .iter()
                .enumerate()
                .map(|(i, (cond, body))| {
                    self.level += 1;
                    let body = self.format_stmts(body);
                    self.level -= 1;
                    format!(
                        "{} {} {{\n{}\n{}}}",
                        if i == 0 { "if" } else { "elif" },
                        self.format_expr(cond),
                        body,
                        self.indentation()
                    )
                })
                .join(" "),
            AstKind::PlusAssignment(lhs, rhs) => format!("{} += {}", lhs, self.format_expr(&rhs)),
            AstKind::MinusAssignment(lhs, rhs) => format!("{} -= {}", lhs, self.format_expr(&rhs)),
            AstKind::TimesAssignment(lhs, rhs) => format!("{} *= {}", lhs, self.format_expr(&rhs)),
            AstKind::DivideAssignment(lhs, rhs) => format!("{} /= {}", lhs, self.format_expr(&rhs)),
            AstKind::FunctionCall(name, args) => format!("{}({})", name, self.format_args(&args)),
            AstKind::FunctionDefinition(name, args, block) => {
                self.level += 1;
                let body = self.format_stmts(&block);
                self.level -= 1;

                format!(
                    "fn {}({}) {{\n{}\n{}}}",
                    name,
                    self.format_args(&args),
                    body,
                    self.indentation()
                )
            }
            AstKind::Expression(expr) => self.format_expr(&expr),
            AstKind::WhileStatement(cond, body) => {
                self.level += 1;
                let body = self.format_stmts(&body);
                self.level -= 1;
                format!(
                    "while {} {{\n{}\n{}}}",
                    self.format_expr(&cond),
                    body,
                    self.indentation()
                )
            }
            AstKind::ClassDefinition(name, members) => {
                let body = members
                    .iter()
                    .map(|member| {
                        self.level += 1;
                        let res = format!(
                            "{}{}",
                            self.indentation(),
                            match member {
                                ClassMember::Field(name, default) => {
                                    if default.kind == ExpressionKind::Null {
                                        format!("var {}", name)
                                    } else {
                                        format!("var {} = {}", name, self.format_expr(default))
                                    }
                                }
                                ClassMember::Function(name, args, body) => {
                                    self.level += 1;
                                    let body = self.format_stmts(body);
                                    self.level -= 1;
                                    format!(
                                        "fn {}({}) {{\n{}\n{}}}",
                                        name,
                                        args.join(", "),
                                        body,
                                        self.indentation()
                                    )
                                }
                                _ => todo!("{:?}", member),
                            }
                        );

                        self.level -= 1;

                        res
                    })
                    .join("\n");
                format!("{} {{\n{}\n}}", name, body)
            }
            _ => todo!("{:?}", ast),
        }
    }

    fn format_stmts(&mut self, stmts: &Vec<AstNode>) -> String {
        stmts
            .iter()
            .map(|stmt| {
                format!(
                    "{}{}{}",
                    self.indentation(),
                    self.format_ast(stmt),
                    self.get_line_ending(stmt)
                )
            })
            .join("\n")
    }

    pub fn format(&mut self) -> String {
        self.format_stmts(&self.prog.stmts)
    }
}

#[cfg(test)]
mod test {
    use super::Formatter;
    use crate::ast::Ast;
    use crate::interpreter::Program;
    use crate::parser::Parser;

    fn format(mut expected: &str) {
        expected = &expected[1..];
        let mut parser = Parser::new();
        parser.set_source("".into(), expected, 0);
        let program = parser.parse().unwrap();
        let actual = Formatter::new(&program).format();
        assert_eq!(actual, expected)
    }

    #[test]
    fn format_while() {
        format(
            r#"
while true {
    print()
}"#,
        )
    }
}
