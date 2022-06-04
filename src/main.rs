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
        Some(path) => run_file(path),
        None => run_prompt(),
    }
}

fn run_file(path: PathBuf) -> Result<()> {
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

fn run(source: &str) -> Result<()> {
    let scanner = scanner::Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;

    // for debugging
    // for token in &tokens {
    //     println!("{:?}", token);
    // }

    let parser = parser::Parser;
    let stmts = parser.parse(tokens)?;

    // let mut printer = AstPrinter;
    // println!("{}", printer.visit_expr(&expr));

    // for debugging
    // println!("{:?}", stmts);

    let interpreter = interpreter::Interpreter::default();
    interpreter.interpret(&stmts)?;

    Ok(())
}
