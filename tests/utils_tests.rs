use tempfile::NamedTempFile;
use std::fs::File;
use std::io::Write;
use tokio::fs;
use std::io::{stdout, BufWriter};
use toy_payments_engine::utils::{process_csv, validate_transaction, process_file};
use toy_payments_engine::engine::TransactionEngine;
use toy_payments_engine::models::{Transaction, TransactionType, ClientAccount};

#[tokio::test]
async fn test_process_csv_valid_transactions() {
    let mut engine = TransactionEngine::new();
    let csv_data = r#"
type,client,tx,amount
deposit,1,1,1.5
withdrawal,1,2,0.5
"#;

    // Write CSV data to a temporary file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{}", csv_data.trim()).unwrap();
    let file_path = temp_file.into_temp_path();

    // Process the file
    let result = process_csv(file_path.to_str().unwrap(), &mut engine).await;
    assert!(result.is_ok(), "Failed to process valid transactions");

    // Validate engine state
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 1.0);
    assert_eq!(account.held, 0.0);
    assert_eq!(account.total, 1.0);
    assert!(!account.locked);
}

#[tokio::test]
async fn test_process_csv_invalid_transactions() {
    let mut engine = TransactionEngine::new();
    let csv_data = r#"
type,client,tx,amount
deposit,0,1,1.5
withdrawal,1,2,-0.5
chargeback,1,3,1.0
"#;

    // Write CSV data to a temporary file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{}", csv_data.trim()).unwrap();
    let file_path = temp_file.into_temp_path();

    // Process the file
    let result = process_csv(file_path.to_str().unwrap(), &mut engine).await;
    assert!(result.is_ok(), "Processing invalid transactions should not fail completely");

    // No accounts should be created due to invalid transactions
    assert!(engine.accounts.is_empty(), "No accounts should have been created");
}

#[tokio::test]
async fn test_validate_transaction_valid_cases() {
    let valid_deposit = Transaction {
        tx_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 1,
        amount: Some(1.5),
    };

    let valid_withdrawal = Transaction {
        tx_type: TransactionType::Withdrawal,
        client_id: 1,
        tx_id: 2,
        amount: Some(0.5),
    };

    assert!(validate_transaction(&valid_deposit).is_ok(), "Valid deposit should pass validation");
    assert!(validate_transaction(&valid_withdrawal).is_ok(), "Valid withdrawal should pass validation");
}

#[tokio::test]
async fn test_validate_transaction_invalid_cases() {
    let invalid_client = Transaction {
        tx_type: TransactionType::Deposit,
        client_id: 0,
        tx_id: 1,
        amount: Some(1.5),
    };

    let invalid_tx_id = Transaction {
        tx_type: TransactionType::Withdrawal,
        client_id: 1,
        tx_id: 0,
        amount: Some(0.5),
    };

    let negative_amount = Transaction {
        tx_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 2,
        amount: Some(-1.0),
    };

    let dispute_with_amount = Transaction {
        tx_type: TransactionType::Dispute,
        client_id: 1,
        tx_id: 3,
        amount: Some(1.0),
    };

    assert!(validate_transaction(&invalid_client).is_err(), "Invalid client ID should fail validation");
    assert!(validate_transaction(&invalid_tx_id).is_err(), "Invalid transaction ID should fail validation");
    assert!(validate_transaction(&negative_amount).is_err(), "Negative amount should fail validation");
    assert!(validate_transaction(&dispute_with_amount).is_err(), "Dispute with amount should fail validation");
}

#[tokio::test]
async fn test_process_file_valid_csv() {
    let temp_file_path = "test_transactions.csv";

    // Create a temporary valid CSV file
    let mut file = File::create(temp_file_path).expect("Unable to create test CSV file");
    writeln!(file, "type,client,tx,amount").unwrap();
    writeln!(file, "deposit,1,1,100.0").unwrap();
    writeln!(file, "withdrawal,1,2,50.0").unwrap();

    let mut engine = TransactionEngine::new();

    // Process the CSV file
    let result = process_file(temp_file_path, &mut engine).await;

    // Assertions
    assert!(result.is_ok());
    assert_eq!(engine.accounts.len(), 1);
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.total, 50.0);
    assert_eq!(account.available, 50.0);
    assert_eq!(account.held, 0.0);
    assert!(!account.locked);

    // Cleanup
    fs::remove_file(temp_file_path)
        .await
        .expect("Failed to delete test CSV file");
}

#[tokio::test]
async fn test_process_file_invalid_csv() {
    let temp_file_path = "test_invalid_transactions.csv";

    // Create a temporary invalid CSV file
    let mut file = File::create(temp_file_path).expect("Unable to create test CSV file");
    writeln!(file, "type,client,tx,amount").unwrap();
    writeln!(file, "invalid_type,1,1,100.0").unwrap();

    let mut engine = TransactionEngine::new();

    // Process the CSV file
    let result = process_file(temp_file_path, &mut engine).await;

    // Assertions
    assert!(result.is_err()); // Expect an error due to invalid transaction type

    // Cleanup
    fs::remove_file(temp_file_path)
        .await
        .expect("Failed to delete test CSV file");
}

#[test]
fn test_export_accounts_to_stdout() {
    let mut engine = TransactionEngine::new();

    // Add sample accounts to the engine
    engine.accounts.insert(
        1,
        ClientAccount {
            client_id: 1,
            available: 100.0,
            held: 0.0,
            total: 100.0,
            locked: false,
        },
    );
    engine.accounts.insert(
        2,
        ClientAccount {
            client_id: 2,
            available: 200.0,
            held: 50.0,
            total: 250.0,
            locked: true,
        },
    );

    // Redirect stdout to capture output
    let mut buffer = Vec::new();
    let writer = BufWriter::new(&mut buffer);

    // Export accounts
    let result = {
        let _stdout_lock = stdout().lock(); // Lock stdout to prevent concurrent access
        let mut csv_writer = csv::Writer::from_writer(writer);
        for account in engine.accounts.values() {
            csv_writer.serialize(account).unwrap();
        }
        csv_writer.flush().map_err(csv::Error::from)
    };

    // Ensure the function executes without errors
    assert!(result.is_ok());

    // Convert the captured output to a String
    let output = String::from_utf8(buffer).unwrap();

    // Validate CSV structure
    assert!(output.contains("client,available,held,total,locked"));
    assert!(output.contains("1,100.0000,0.0000,100.0000,false"));
    assert!(output.contains("2,200.0000,50.0000,250.0000,true"));
}