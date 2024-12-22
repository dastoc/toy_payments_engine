use std::env;
use crate::engine::TransactionEngine;
use crate::utils::{export_accounts_to_stdout, process_file};

mod engine;
mod models;
mod utils;

pub async fn run_program(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return Err("Missing input file".into());
    }

    let input_file = &args[1];
    let mut engine = TransactionEngine::new();

    if let Err(e) = process_file(input_file, &mut engine).await {
        eprintln!("Error processing file: {}", e);
        return Err(e.into());
    }

    if let Err(e) = export_accounts_to_stdout(&engine) {
        eprintln!("Error exporting accounts: {}", e);
        return Err(e.into());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    run_program(args).await
}