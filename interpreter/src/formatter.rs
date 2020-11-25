use std::fmt::Display;

use itertools::Itertools;

use super::{ast::Ast, expression::Expression, interpreter::Program};

pub struct Formatter<'a> {
    prog: &'a Program,
    level: usize,
}

impl<'a> Formatter<'a> {
    pub fn new(prog: &'a Program) -> Self {
        Self { prog, level: 0 }
    }

    fn format_expr(&mut self, expr: &Expression) -> String {
        expr.to_string()
    }

    fn format_args<T: Display>(&mut self, args: &Vec<T>) -> String {
        args.iter().map(|a| a.to_string()).join(", ")
    }

    fn get_line_ending(&self, ast: &Ast) -> String {
        match ast {
            Ast::FunctionDefinition(_, _, _) => "",
            _ => ";",
        }
        .into()
    }

    fn format_ast(&mut self, ast: &Ast) -> String {
        match ast {
            Ast::VariableDefinition(name, value) => {
                format!("let {} = {}", name, self.format_expr(value))
            }
            Ast::VariableAssignment(_, _) => todo!(),
            Ast::IfStatement(_) => todo!(),
            Ast::FunctionCall(name, args) => format!("{}({})", name, self.format_args(args)),
            Ast::FunctionDefinition(name, args, block) => {
                self.level += 1;

                let result = format!(
                    "fn {}({}) {{\n{}\n}}",
                    name,
                    self.format_args(args),
                    self.format_stmts(block)
                );

                self.level -= 1;

                result
            }
            _ => todo!(),
        }
    }

    fn format_stmts(&mut self, stmts: &Vec<Ast>) -> String {
        stmts
            .iter()
            .map(|stmt| {
                format!(
                    "{}{}{}",
                    "    ".repeat(self.level),
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
