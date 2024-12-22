use std::io;
use std::io::BufWriter;
use csv_async::AsyncReaderBuilder;
use tokio_util::compat::TokioAsyncReadCompatExt;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use crate::engine::TransactionEngine;
use crate::models::{Transaction, TransactionType};

/// Stream transactions from the CSV file and process them.
pub async fn process_csv(
    file_path: &str,
    engine: &mut TransactionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = tokio::fs::File::open(file_path).await?;
    let file = file.compat();

    let mut reader = AsyncReaderBuilder::new()
        .trim(csv_async::Trim::All)
        .create_reader(file);

    let mut records = reader.records();

    let metadata = tokio::fs::metadata(file_path).await?;
    let total_zise = metadata.len();

    let progress_bar = ProgressBar::new(total_zise);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .progress_chars("#>- ")
            .template("{msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({pos}/{len})")?
    );
    progress_bar.set_message("Processing CSV");

    while let Some(result) = records.next().await {
        match result {
            Ok(record) => {
                // Deserialize the record into a Transaction
                let transaction: Transaction = record.deserialize(None)?;

                // Validate the transaction
                if let Err(e) = validate_transaction(&transaction) {
                    eprintln!("Invalid transaction: {}: {:?}", e, transaction);
                    continue; // Skip invalid transactions
                }

                // Process the valid transaction
                if let Err(e) = engine.handle_transaction(transaction) {
                    eprintln!("Error processing transaction: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading CSV transaction: {}", e);
            }
        }
         // Increment progress bar for each record processed
         progress_bar.inc(1);
    }

    progress_bar.finish_with_message("Processing complete");
    Ok(())
}

/// Validate a transaction's fields.
pub fn validate_transaction(transaction: &Transaction) -> Result<(), String> {
    // Validate client_id
    if transaction.client_id == 0 {
        return Err("Client ID must be greater than 0".into());
    }

    // Validate tx_id
    if transaction.tx_id == 0 {
        return Err("Transaction ID must be greater than 0".into());
    }

    // Validate fields based on transaction type
    match transaction.tx_type {
        TransactionType::Deposit | TransactionType::Withdrawal => {
            // Deposit and Withdrawal must have a valid amount
            if let Some(amount) = transaction.amount {
                if amount < 0.0 {
                    return Err(format!("{:?} amount must be non-negative", transaction));
                }
            } else {
                return Err(format!("{:?} transaction requires an amount", transaction));
            }
        }
        TransactionType::Dispute | TransactionType::Resolve | TransactionType::Chargeback => {
            // Dispute, Resolve, and Chargeback must not have an amount
            if transaction.amount.is_some() {
                return Err(format!(
                    "{:?} transaction must not have an amount",
                    transaction
                ));
            }
        }
    }

    Ok(())
}

/// Processes a CSV file and updates the transaction engine.
pub async fn process_file(
    input_file: &str,
    engine: &mut TransactionEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    process_csv(input_file, engine).await
}

/// Exports accounts to stdout.
pub fn export_accounts_to_stdout(engine: &TransactionEngine) -> Result<(), csv::Error> {
    let stdout = io::stdout();
    let writer = BufWriter::new(stdout.lock());
    let mut csv_writer = csv::Writer::from_writer(writer);
        
    for account in engine.accounts.values() {
        csv_writer.serialize(account)?;
    }

    csv_writer.flush().map_err(csv::Error::from)
}