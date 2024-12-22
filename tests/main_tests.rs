use toy_payments_engine::engine::TransactionEngine;
use toy_payments_engine::models::ClientAccount;
use toy_payments_engine::utils::{export_accounts_to_stdout, process_file};

#[tokio::test]
async fn test_process_file_with_valid_data() {
    let input_data = r#"type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,50.0
"#;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), input_data).unwrap();

    let mut engine = TransactionEngine::new();
    let result = process_file(temp_file.path().to_str().unwrap(), &mut engine).await;

    assert!(result.is_ok());

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 50.0);
    assert_eq!(account.total, 50.0);
    assert_eq!(account.held, 0.0);
}

#[tokio::test]
async fn test_process_file_with_invalid_data() {
    let input_data = r#"type,client,tx,amount
deposit,1,1,-100.0
withdrawal,1,2,50.0
"#;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), input_data).unwrap();

    let mut engine = TransactionEngine::new();
    let result = process_file(temp_file.path().to_str().unwrap(), &mut engine).await;

    assert!(result.is_ok());

    // Verify no accounts were created due to invalid data
    assert!(engine.accounts.is_empty());
}

#[tokio::test]
async fn test_process_file_with_partial_failures() {
    let input_data = r#"type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,150.0
"#;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), input_data).unwrap();

    let mut engine = TransactionEngine::new();
    let result = process_file(temp_file.path().to_str().unwrap(), &mut engine).await;

    assert!(result.is_ok());

    // Verify only valid transactions were processed
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 100.0); // Withdrawal failed due to insufficient funds
    assert_eq!(account.total, 100.0);
    assert_eq!(account.held, 0.0);
}

#[tokio::test]
async fn test_export_accounts_to_stdout() {
    let mut engine = TransactionEngine::new();
    engine.accounts.insert(
        1,
        ClientAccount {
            client_id: 1,
            available: 50.0,
            held: 0.0,
            total: 50.0,
            locked: false,
        },
    );

    let output = std::panic::catch_unwind(|| {
        let _ = export_accounts_to_stdout(&engine);
    });

    assert!(output.is_ok());
}