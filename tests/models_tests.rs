use toy_payments_engine::models::{TransactionType, Transaction, ClientAccount};
use std::str::FromStr;
use serde_json;

// Utility function for floating-point comparison with precision
fn assert_float_eq(a: f64, b: f64, precision: usize) {
    let factor = 10f64.powi(precision as i32);
    assert!((a * factor).round() == (b * factor).round(), "Floats not equal: {} != {}", a, b);
}

#[test]
fn test_transaction_type_from_str() {
    assert_eq!(TransactionType::from_str("deposit").unwrap(), TransactionType::Deposit);
    assert_eq!(TransactionType::from_str("withdrawal").unwrap(), TransactionType::Withdrawal);
    assert_eq!(TransactionType::from_str("dispute").unwrap(), TransactionType::Dispute);
    assert_eq!(TransactionType::from_str("resolve").unwrap(), TransactionType::Resolve);
    assert_eq!(TransactionType::from_str("chargeback").unwrap(), TransactionType::Chargeback);

    assert!(TransactionType::from_str("invalid").is_err());
}

#[test]
fn test_transaction_deserialization() {
    let json_data = r#"{
        "type": "deposit",
        "client": 1,
        "tx": 100,
        "amount": 10.1234
    }"#;

    let transaction: Transaction = serde_json::from_str(json_data).unwrap();
    assert_eq!(transaction.tx_type, TransactionType::Deposit);
    assert_eq!(transaction.client_id, 1);
    assert_eq!(transaction.tx_id, 100);
    assert_float_eq(transaction.amount.unwrap(), 10.1234, 4);
}

#[test]
fn test_transaction_deserialization_invalid_data() {
    // Invalid client ID
    let json_data = r#"{
        "type": "deposit",
        "client": 0,
        "tx": 100,
        "amount": 10.1234
    }"#;

    let transaction: Result<Transaction, serde_json::Error> = serde_json::from_str(json_data);
    assert!(transaction.is_ok(), "Client ID validation is handled elsewhere");

    // Invalid transaction ID
    let json_data = r#"{
        "type": "deposit",
        "client": 1,
        "tx": 0,
        "amount": 10.1234
    }"#;

    let transaction: Result<Transaction, serde_json::Error> = serde_json::from_str(json_data);
    assert!(transaction.is_ok(), "Transaction ID validation is handled elsewhere");

    // Invalid amount(negative)
    let json_data = r#"{
        "type": "deposit",
        "client": 1,
        "tx": 100,
        "amount": -10.1234
    }"#;

    let transaction: Result<Transaction, serde_json::Error> = serde_json::from_str(json_data);
    assert!(transaction.is_ok(), "Amount validation is handled elsewhere");
}

#[test]
fn test_client_account_serialization() {
    let account = ClientAccount {
        client_id: 1,
        available: 1.123456,
        held: 0.987654,
        total: 2.11111,
        locked: false,
    };

    let serialized = serde_json::to_string(&account).unwrap();
    let expected =r#"{"client":1,"available":"1.1235","held":"0.9877","total":"2.1111","locked":false}"#;

    assert_eq!(serialized, expected);
}

#[test]
fn test_client_account_precision() {
    let account = ClientAccount {
        client_id: 1,
        available: 1.123456,
        held: 0.987654,
        total: 2.11111,
        locked: false,
    };

    assert_float_eq(account.available, 1.1235, 4);
    assert_float_eq(account.held, 0.9877, 4);
    assert_float_eq(account.total, 2.1111, 4);
}