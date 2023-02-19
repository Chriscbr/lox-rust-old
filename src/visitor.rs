use crate::{
    expr::{Assign, Binary, Call, Expr, Grouping, Literal, Logical, Unary, Variable},
    stmt::{Block, Expression, Function, If, Print, Return, Stmt, Var, While},
};

pub trait ExprVisitor {
    type ExprResult;
    fn visit_expr(&mut self, expr: &Expr) -> Self::ExprResult {
        match expr {
            Expr::Assign(assign) => self.visit_expr_assign(assign),
            Expr::Binary(binary) => self.visit_expr_binary(binary),
            Expr::Call(call) => self.visit_expr_call(call),
            Expr::Grouping(grouping) => self.visit_expr_grouping(grouping),
            Expr::Literal(literal) => self.visit_expr_literal(literal),
            Expr::Logical(logical) => self.visit_expr_logical(logical),
            Expr::Variable(variable) => self.visit_expr_variable(variable),
            Expr::Unary(unary) => self.visit_expr_unary(unary),
        }
    }
    fn visit_expr_assign(&mut self, assign: &Assign) -> Self::ExprResult;
    fn visit_expr_binary(&mut self, binary: &Binary) -> Self::ExprResult;
    fn visit_expr_call(&mut self, call: &Call) -> Self::ExprResult;
    fn visit_expr_grouping(&mut self, grouping: &Grouping) -> Self::ExprResult;
    fn visit_expr_literal(&mut self, literal: &Literal) -> Self::ExprResult;
    fn visit_expr_logical(&mut self, logical: &Logical) -> Self::ExprResult;
    fn visit_expr_variable(&mut self, variable: &Variable) -> Self::ExprResult;
    fn visit_expr_unary(&mut self, unary: &Unary) -> Self::ExprResult;
}

pub trait StmtVisitor {
    type StmtResult;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::StmtResult {
        match stmt {
            Stmt::Block(block) => self.visit_stmt_block(block),
            Stmt::Expression(expression) => self.visit_stmt_expression(expression),
            Stmt::Function(function) => self.visit_stmt_function(function),
            Stmt::If(if_) => self.visit_stmt_if(if_),
            Stmt::Print(print) => self.visit_stmt_print(print),
            Stmt::Return(return_) => self.visit_stmt_return(return_),
            Stmt::Var(var) => self.visit_stmt_var(var),
            Stmt::While(while_) => self.visit_stmt_while(while_),
        }
    }
    fn visit_stmt_block(&mut self, block: &Block) -> Self::StmtResult;
    fn visit_stmt_expression(&mut self, expression: &Expression) -> Self::StmtResult;
    fn visit_stmt_function(&mut self, function: &Function) -> Self::StmtResult;
    fn visit_stmt_if(&mut self, if_: &If) -> Self::StmtResult;
    fn visit_stmt_print(&mut self, print: &Print) -> Self::StmtResult;
    fn visit_stmt_return(&mut self, return_: &Return) -> Self::StmtResult;
    fn visit_stmt_var(&mut self, var: &Var) -> Self::StmtResult;
    fn visit_stmt_while(&mut self, while_: &While) -> Self::StmtResult;
}

pub trait Visit<'ast> {
    fn visit_expr(&mut self, e: &'ast Expr) {
        visit_expr(self, e);
    }
    fn visit_expr_assign(&mut self, e: &'ast Assign) {
        visit_expr_assign(self, e);
    }
    fn visit_expr_binary(&mut self, e: &'ast Binary) {
        visit_expr_binary(self, e);
    }
    fn visit_expr_call(&mut self, e: &'ast Call) {
        visit_expr_call(self, e);
    }
    fn visit_expr_grouping(&mut self, e: &'ast Grouping) {
        visit_expr_grouping(self, e);
    }
    fn visit_expr_literal(&mut self, e: &'ast Literal) {
        visit_expr_literal(self, e);
    }
    fn visit_expr_logical(&mut self, e: &'ast Logical) {
        visit_expr_logical(self, e);
    }
    fn visit_expr_variable(&mut self, e: &'ast Variable) {
        visit_expr_variable(self, e);
    }
    fn visit_expr_unary(&mut self, e: &'ast Unary) {
        visit_expr_unary(self, e);
    }
    fn visit_stmt(&mut self, s: &'ast Stmt) {
        visit_stmt(self, s);
    }
    fn visit_stmt_block(&mut self, s: &'ast Block) {
        visit_stmt_block(self, s);
    }
    fn visit_stmt_expression(&mut self, s: &'ast Expression) {
        visit_stmt_expression(self, s);
    }
    fn visit_stmt_function(&mut self, s: &'ast Function) {
        visit_stmt_function(self, s);
    }
    fn visit_stmt_if(&mut self, s: &'ast If) {
        visit_stmt_if(self, s);
    }
    fn visit_stmt_print(&mut self, s: &'ast Print) {
        visit_stmt_print(self, s);
    }
    fn visit_stmt_return(&mut self, s: &'ast Return) {
        visit_stmt_return(self, s);
    }
    fn visit_stmt_var(&mut self, s: &'ast Var) {
        visit_stmt_var(self, s);
    }
    fn visit_stmt_while(&mut self, s: &'ast While) {
        visit_stmt_while(self, s);
    }
}

pub fn visit_expr<'ast, V>(v: &mut V, node: &'ast Expr)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        Expr::Assign(assign) => {
            v.visit_expr_assign(assign);
        }
        Expr::Binary(binary) => {
            v.visit_expr_binary(binary);
        }
        Expr::Call(call) => {
            v.visit_expr_call(call);
        }
        Expr::Grouping(grouping) => {
            v.visit_expr_grouping(grouping);
        }
        Expr::Literal(literal) => {
            v.visit_expr_literal(literal);
        }
        Expr::Logical(logical) => {
            v.visit_expr_logical(logical);
        }
        Expr::Variable(variable) => {
            v.visit_expr_variable(variable);
        }
        Expr::Unary(unary) => {
            v.visit_expr_unary(unary);
        }
    }
}

pub fn visit_expr_assign<'ast, V>(v: &mut V, node: &'ast Assign)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.value);
}

pub fn visit_expr_binary<'ast, V>(v: &mut V, node: &'ast Binary)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.left);
    v.visit_expr(&node.right);
}

pub fn visit_expr_call<'ast, V>(v: &mut V, node: &'ast Call)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.callee);
    for arg in &node.arguments {
        v.visit_expr(arg);
    }
}

pub fn visit_expr_grouping<'ast, V>(v: &mut V, node: &'ast Grouping)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.expression);
}

pub fn visit_expr_literal<'ast, V>(_: &mut V, _: &'ast Literal)
where
    V: Visit<'ast> + ?Sized,
{
}

pub fn visit_expr_logical<'ast, V>(v: &mut V, node: &'ast Logical)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.left);
    v.visit_expr(&node.right);
}

pub fn visit_expr_variable<'ast, V>(_: &mut V, _: &'ast Variable)
where
    V: Visit<'ast> + ?Sized,
{
}

pub fn visit_expr_unary<'ast, V>(v: &mut V, node: &'ast Unary)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.right);
}

pub fn visit_stmt<'ast, V>(v: &mut V, node: &'ast Stmt)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        Stmt::Block(block) => {
            v.visit_stmt_block(block);
        }
        Stmt::Expression(expression) => {
            v.visit_stmt_expression(expression);
        }
        Stmt::Function(function) => {
            v.visit_stmt_function(function);
        }
        Stmt::If(if_) => {
            v.visit_stmt_if(if_);
        }
        Stmt::Print(print) => {
            v.visit_stmt_print(print);
        }
        Stmt::Return(return_) => {
            v.visit_stmt_return(return_);
        }
        Stmt::Var(var) => {
            v.visit_stmt_var(var);
        }
        Stmt::While(while_) => {
            v.visit_stmt_while(while_);
        }
    }
}

pub fn visit_stmt_block<'ast, V>(v: &mut V, node: &'ast Block)
where
    V: Visit<'ast> + ?Sized,
{
    for stmt in &node.statements {
        v.visit_stmt(stmt);
    }
}

pub fn visit_stmt_expression<'ast, V>(v: &mut V, node: &'ast Expression)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.expression);
}

pub fn visit_stmt_function<'ast, V>(v: &mut V, node: &'ast Function)
where
    V: Visit<'ast> + ?Sized,
{
    for stmt in &node.body {
        v.visit_stmt(&stmt);
    }
}

pub fn visit_stmt_if<'ast, V>(v: &mut V, node: &'ast If)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.condition);
    v.visit_stmt(&node.then_branch);
    if let Some(else_branch) = &node.else_branch {
        v.visit_stmt(else_branch);
    }
}

pub fn visit_stmt_print<'ast, V>(v: &mut V, node: &'ast Print)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.expression);
}

pub fn visit_stmt_return<'ast, V>(v: &mut V, node: &'ast Return)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.value);
}

pub fn visit_stmt_var<'ast, V>(v: &mut V, node: &'ast Var)
where
    V: Visit<'ast> + ?Sized,
{
    if let Some(initializer) = &node.initializer {
        v.visit_expr(initializer);
    }
}

pub fn visit_stmt_while<'ast, V>(v: &mut V, node: &'ast While)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.condition);
    v.visit_stmt(&node.body);
}
