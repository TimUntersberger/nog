use std::fmt::Display;

use itertools::Itertools;

use super::{
    ast::Ast, ast::ClassMember, expression::Expression, interpreter::Program, operator::Operator,
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
        match expr {
            Expression::Null => "null".into(),
            Expression::StringLiteral(text) => format!(
                "\"{}\"",
                text.replace("\r", "\\r")
                    .replace("\n", "\\n")
                    .replace("\t", "\\t")
                    .replace("\"", "\\\"")
            ),
            Expression::Identifier(text)
            | Expression::ClassIdentifier(text)
            | Expression::NumberLiteral(text)
            | Expression::HexLiteral(text)
            | Expression::BooleanLiteral(text) => text.clone(),
            Expression::ArrayLiteral(items) => format!(
                "[{}]",
                items
                    .into_iter()
                    .map(|expr| self.format_expr(expr))
                    .join(", ")
            ),
            Expression::ObjectLiteral(fields) => {
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
            Expression::ArrowFunction(args, body) => {
                if body.len() == 1 {
                    let body = body.get(0).unwrap();
                    match body {
                        Ast::ReturnStatement(expr) | Ast::Expression(expr) => {
                            let body = self.format_expr(expr);
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
            Expression::ClassInstantiation(name, fields) => format!(
                "{}{{{}}}",
                name,
                fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_expr(v)))
                    .join("\n")
            ),
            Expression::PreOp(op, expr) => format!("{}{}", op.to_string(), self.format_expr(expr)),
            Expression::BinaryOp(lhs, op, rhs) => match op {
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
            Expression::PostOp(lhs, op, value) => match op {
                Operator::Call => format!("{}({})", self.format_expr(lhs), {
                    if let Expression::ArrayLiteral(exprs) = value.as_ref().unwrap().as_ref() {
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

    fn get_line_ending(&self, ast: &Ast) -> String {
        match ast {
            // Ast::FunctionDefinition(_, _, _) => "",
            _ => "",
        }
        .into()
    }

    fn indentation(&self) -> String {
        "    ".repeat(self.level)
    }

    fn format_ast(&mut self, ast: &Ast) -> String {
        match ast {
            Ast::VariableDefinition(name, value) => {
                format!("var {} = {}", name, self.format_expr(value))
            }
            Ast::VariableAssignment(name, value) => {
                format!("{} = {}", name, self.format_expr(value))
            }
            Ast::Documentation(lines) => lines
                .iter()
                .map(|line| format!("///{}", line))
                .join(&format!("\n{}", self.indentation())),
            Ast::Comment(lines) => lines
                .iter()
                .map(|line| format!("//{}", line))
                .join(&format!("\n{}", self.indentation())),
            Ast::BreakStatement => "break".into(),
            Ast::ReturnStatement(expr) => format!("return {}", self.format_expr(expr)),
            Ast::ExportStatement(stmt) => format!("export {}", self.format_ast(stmt)),
            Ast::ImportStatement(path) => format!("import {}", path),
            Ast::IfStatement(branches) => branches
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
            Ast::PlusAssignment(lhs, rhs) => format!("{} += {}", lhs, self.format_expr(rhs)),
            Ast::MinusAssignment(lhs, rhs) => format!("{} -= {}", lhs, self.format_expr(rhs)),
            Ast::TimesAssignment(lhs, rhs) => format!("{} *= {}", lhs, self.format_expr(rhs)),
            Ast::DivideAssignment(lhs, rhs) => format!("{} /= {}", lhs, self.format_expr(rhs)),
            Ast::FunctionCall(name, args) => format!("{}({})", name, self.format_args(&args)),
            Ast::FunctionDefinition(name, args, block) => {
                self.level += 1;
                let body = self.format_stmts(block);
                self.level -= 1;

                format!(
                    "fn {}({}) {{\n{}\n{}}}",
                    name,
                    self.format_args(args),
                    body,
                    self.indentation()
                )
            }
            Ast::Expression(expr) => self.format_expr(expr),
            Ast::WhileStatement(cond, body) => {
                self.level += 1;
                let body = self.format_stmts(body);
                self.level -= 1;
                format!(
                    "while {} {{\n{}\n{}}}",
                    self.format_expr(cond),
                    body,
                    self.indentation()
                )
            }
            Ast::ClassDefinition(name, members) => {
                let body = members
                    .iter()
                    .map(|member| {
                        self.level += 1;
                        let res = format!(
                            "{}{}",
                            self.indentation(),
                            match member {
                                ClassMember::Field(name, default) => {
                                    if default == &Expression::Null {
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

    fn format_stmts(&mut self, stmts: &Vec<Ast>) -> String {
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
