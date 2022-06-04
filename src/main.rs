use std::{
    fs::read_to_string,
    io::{stdin, stdout, BufRead, BufReader, Write},
    path::PathBuf,
};

mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;
mod visitor;

use anyhow::{Context, Result};
use structopt::StructOpt;

/// Run a lox script.
#[derive(StructOpt)]
struct Cli {
    /// Path to a lox file.
    #[structopt(parse(from_os_str))]
    script: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::from_args();

    match args.script {
        Some(path) => run_file(path).map(|_| ()),
        None => run_prompt(),
    }
}

fn run_file(path: PathBuf) -> Result<String> {
    let contents =
        read_to_string(&path).with_context(|| format!("could not read file {:?}", &path))?;
    run(&contents)
}

fn run_prompt() -> Result<()> {
    let mut reader = BufReader::new(stdin());
    loop {
        let mut buffer = String::new();
        print!("> ");
        stdout().flush().with_context(|| "could not flush stdout")?;
        reader.read_line(&mut buffer)?;
        if buffer.is_empty() {
            return Ok(());
        };
        run(&buffer)?;
    }
}

fn run(source: &str) -> Result<String> {
    let scanner = scanner::Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;

    // for debugging
    // for token in &tokens {
    //     println!("{:?}", token);
    // }

    let mut parser = parser::Parser::new(tokens);
    let stmts = parser.parse()?;

    // let mut printer = AstPrinter;
    // println!("{}", printer.visit_expr(&expr));

    // for debugging
    // println!("{:?}", stmts);

    let interpreter = interpreter::Interpreter::default();
    let stdout = interpreter.interpret(&stmts)?;

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integ_tests() {
        assert_eq!(
            run_file("examples/fibonacci.lox".into()).unwrap(),
            r#"0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
610
987
1597
2584
4181
6765
"#
            .to_owned()
        );

        assert_eq!(
            run_file("examples/stmts.lox".into()).unwrap(),
            r#"one
true
3
"#
            .to_owned()
        );

        assert_eq!(
            run_file("examples/scopes.lox".into()).unwrap(),
            r#"inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
"#
            .to_owned()
        );

        assert_eq!(
            run_file("examples/scopes2.lox".into()).unwrap(),
            r#"3
3
1
3
"#
            .to_owned()
        );

        assert_eq!(
            run_file("examples/variables.lox".into()).unwrap(),
            r#"3
2
"#
            .to_owned()
        );
    }
}
