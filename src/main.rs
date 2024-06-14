use eyre::Result;
use log_parser::runner::Runner;
use std::{env, io::stdout};

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_log_file>", args[0]);
        return Ok(());
    }

    let log_file = &args[1];

    let stdout_writer = stdout();

    let mut runner = Runner::builder()
        .with_log_file(log_file)
        .with_stdout_writer(stdout_writer)
        .build()?;

    runner.run()?;

    Ok(())
}
