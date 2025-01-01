mod commands;

use crate::commands::Args;
use clap::Parser;
use interpreter::processor::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let data: Vec<u8> = fs::read(args.path.clone()).map_err(|err| {
        format!(
            "Error reading input file at {}: {}",
            args.path.to_str().unwrap_or("<Invalid Path>"),
            err
        )
    })?;

    let mut proc = Processor::new(data).map_err(|err| {
        format!(
            "Error occured loading program at path {}: {}",
            args.path.display(),
            err
        )
    })?;

    loop {
        proc.step().map_err(|err| format!("{}", err))?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
