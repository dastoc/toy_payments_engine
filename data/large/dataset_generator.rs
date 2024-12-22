//! ```cargo
//! [dependencies]
//! csv = "1.3.1"
//! ```

use csv::Writer;
use std::env;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <num_clients> <transactions_per_client> <output_file>", args[0]);
        std::process::exit(1);
    }

    // Parse arguments
    let num_clients: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error: num_clients must be a positive integer.");
        std::process::exit(1);
    });

    let transactions_per_client: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error: transactions_per_client must be a positive integer.");
        std::process::exit(1);
    });

    let output_file = &args[3];

    // Create the dataset
    let file = File::create(output_file)?;
    let mut writer = Writer::from_writer(file);

    // Write header
    writer.write_record(&["type", "client", "tx", "amount"])?;

    let mut tx_id = 1;
    for client_id in 1..=num_clients {
        for _ in 0..transactions_per_client {
            writer.write_record(&["deposit", &client_id.to_string(), &tx_id.to_string(), "100.0"])?;
            writer.write_record(&["withdrawal", &client_id.to_string(), &(tx_id + 1).to_string(), "50.0"])?;
            tx_id += 2;
        }
    }

    writer.flush()?;
    println!("Dataset generated successfully: {}", output_file);
    Ok(())
}