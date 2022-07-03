use std::{
    fs::read_to_string,
    io::{stdin, stdout, BufRead, BufReader, Write},
    path::PathBuf,
};

mod ast_printer;
mod cursor;
mod env;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;
mod visitor;

use anyhow::{Context, Result};

pub fn run_file(path: PathBuf) -> Result<String> {
    let contents =
        read_to_string(&path).with_context(|| format!("could not read file {:?}", &path))?;
    run(&contents)
}

pub fn run_prompt() -> Result<()> {
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

pub fn run(source: &str) -> Result<String> {
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
    fn unicode_support() {
        assert_eq!(run(r#"print "Hello, 世界";"#).unwrap(), "Hello, 世界\n");
    }

    #[test]
    fn integ_fibonacci() {
        assert_eq!(
            run_file("examples/fibonacci.lox".into()).unwrap(),
            vec![
                "0", "1", "1", "2", "3", "5", "8", "13", "21", "34", "55", "89", "144", "233",
                "377", "610", "987", "1597", "2584", "4181", "6765", ""
            ]
            .join("\n")
        );
    }

    #[test]
    fn integ_fibonacci_rec() {
        assert_eq!(
            run_file("examples/fibonacci-rec.lox".into()).unwrap(),
            vec![
                "0", "1", "1", "2", "3", "5", "8", "13", "21", "34", "55", "89", "144", "233",
                "377", "610", "987", "1597", "2584", "4181", "6765", ""
            ]
            .join("\n")
        );
    }

    #[test]
    fn integ_stmts() {
        assert_eq!(
            run_file("examples/stmts.lox".into()).unwrap(),
            vec!["one", "true", "3", ""].join("\n")
        );
    }

    #[test]
    fn integ_scopes() {
        assert_eq!(
            run_file("examples/scopes.lox".into()).unwrap(),
            vec![
                "inner a", "outer b", "global c", "outer a", "outer b", "global c", "global a",
                "global b", "global c", ""
            ]
            .join("\n")
        );
    }

    #[test]
    fn integ_scopes2() {
        assert_eq!(
            run_file("examples/scopes2.lox".into()).unwrap(),
            vec!["3", "3", "1", "3", ""].join("\n")
        );
    }

    #[test]
    fn integ_scopes3() {
        assert_eq!(
            run_file("examples/scopes3.lox".into()).unwrap(),
            vec!["local", ""].join("\n")
        );
    }

    #[test]
    fn integ_scopes4() {
        assert_eq!(
            run_file("examples/scopes4.lox".into()).unwrap(),
            vec!["global", "global", ""].join("\n")
        );
    }

    #[test]
    fn integ_variables() {
        assert_eq!(
            run_file("examples/variables.lox".into()).unwrap(),
            vec!["3", "2", ""].join("\n")
        );
    }

    #[test]
    fn integ_functions1() {
        assert_eq!(
            run_file("examples/functions1.lox".into()).unwrap(),
            vec!["6", ""].join("\n")
        );
    }

    #[test]
    fn integ_functions2() {
        assert_eq!(
            run_file("examples/functions2.lox".into()).unwrap(),
            vec!["1", "2", "3", ""].join("\n")
        );
    }

    #[test]
    fn integ_functions3() {
        assert_eq!(
            run_file("examples/functions3.lox".into()).unwrap(),
            vec!["Hi, Dear Reader!", ""].join("\n")
        );
    }

    #[test]
    fn integ_counter() {
        assert_eq!(
            run_file("examples/counter.lox".into()).unwrap(),
            vec!["1", "2", "1", "3", ""].join("\n")
        );
    }
}
