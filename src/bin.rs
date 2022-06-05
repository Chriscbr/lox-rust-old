use anyhow::Result;
use lox_lib::{run_file, run_prompt};
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
