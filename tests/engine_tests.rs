use toy_payments_engine::engine::TransactionEngine;
use toy_payments_engine::models::{TransactionType, Transaction, ClientAccount};

#[test]
fn test_engine_initialization() {
    let engine = TransactionEngine::new();
    assert!(engine.accounts.is_empty());
    assert!(engine.transactions.is_empty());
}

#[test]
fn test_handle_deposit() {
    let mut engine = TransactionEngine::new();
    let transaction = Transaction {
        tx_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 1,
        amount: Some(100.0),
    };
    assert!(engine.handle_transaction(transaction).is_ok());

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 100.0);
    assert_eq!(account.held, 0.0);
    assert_eq!(account.total, 100.0);
    assert!(!account.locked);
}

#[test]
fn test_deposit_to_new_account() {
    let mut engine = TransactionEngine::new();
    let deposit = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Deposit,
        amount: Some(100.0),
    };

    assert!(engine.handle_transaction(deposit).is_ok());
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 100.0);
    assert_eq!(account.total, 100.0);
    assert_eq!(account.held, 0.0);
}

#[test]
fn test_handle_deposit_with_invalid_amount() {
    let mut engine = TransactionEngine::new();
    let transaction = Transaction {
        tx_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 1,
        amount: None,
    };
    assert!(engine.handle_transaction(transaction).is_err());
}

#[test]
fn test_deposit_negative_amount() {
    let mut engine = TransactionEngine::new();
    let deposit = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Deposit,
        amount: Some(-50.0),
    };

    let result = engine.handle_transaction(deposit);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Deposit amount must be positive");
}

#[test]
fn test_handle_withdrawal() {
    let mut engine = TransactionEngine::new();
    let deposit = Transaction {
        tx_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 1,
        amount: Some(100.0),
    };
    assert!(engine.handle_transaction(deposit).is_ok());

    let withdrawal = Transaction {
        tx_type: TransactionType::Withdrawal,
        client_id: 1,
        tx_id: 2,
        amount: Some(50.0),
    };
    assert!(engine.handle_transaction(withdrawal).is_ok());

    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 50.0);
    assert_eq!(account.held, 0.0);
    assert_eq!(account.total, 50.0);
    assert!(!account.locked);
}

#[test]
fn test_withdrawal_from_existing_account() {
    let mut engine = TransactionEngine::new();
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

    let withdrawal = Transaction {
        tx_id: 2,
        client_id: 1,
        tx_type: TransactionType::Withdrawal,
        amount: Some(50.0),
    };

    assert!(engine.handle_transaction(withdrawal).is_ok());
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 50.0);
    assert_eq!(account.total, 50.0);
}

#[test]
fn test_withdrawal_insufficient_funds() {
    let mut engine = TransactionEngine::new();
    engine.accounts.insert(
        1,
        ClientAccount {
            client_id: 1,
            available: 30.0,
            held: 0.0,
            total: 30.0,
            locked: false,
        },
    );

    let withdrawal = Transaction {
        tx_id: 2,
        client_id: 1,
        tx_type: TransactionType::Withdrawal,
        amount: Some(50.0),
    };

    let result = engine.handle_transaction(withdrawal);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Insufficient funds: Available = 30, Withdrawal amount = 50");
}

#[test]
fn test_dispute_transaction() {
    let mut engine = TransactionEngine::new();
    let deposit = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Deposit,
        amount: Some(100.0),
    };
    engine.handle_transaction(deposit).unwrap();

    let dispute = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Dispute,
        amount: None,
    };

    assert!(engine.handle_transaction(dispute).is_ok());
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 100.0);
}

#[test]
fn test_resolve_dispute() {
    let mut engine = TransactionEngine::new();
    let deposit = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Deposit,
        amount: Some(100.0),
    };
    engine.handle_transaction(deposit).unwrap();

    let dispute = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Dispute,
        amount: None,
    };
    engine.handle_transaction(dispute).unwrap();

    let resolve = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Resolve,
        amount: None,
    };

    assert!(engine.handle_transaction(resolve).is_ok());
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, 100.0);
    assert_eq!(account.held, 0.0);
}

#[test]
fn test_chargeback() {
    let mut engine = TransactionEngine::new();
    let deposit = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Deposit,
        amount: Some(100.0),
    };
    engine.handle_transaction(deposit).unwrap();

    let dispute = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Dispute,
        amount: None,
    };
    engine.handle_transaction(dispute).unwrap();

    let chargeback = Transaction {
        tx_id: 1,
        client_id: 1,
        tx_type: TransactionType::Chargeback,
        amount: None,
    };

    assert!(engine.handle_transaction(chargeback).is_ok());
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.total, 0.0);
    assert_eq!(account.held, 0.0);
    assert!(account.locked);
}

#[test]
fn test_transaction_on_locked_account() {
    let mut engine = TransactionEngine::new();
    engine.accounts.insert(
        1,
        ClientAccount {
            client_id: 1,
            available: 100.0,
            held: 0.0,
            total: 100.0,
            locked: true,
        },
    );

    let deposit = Transaction {
        tx_id: 2,
        client_id: 1,
        tx_type: TransactionType::Deposit,
        amount: Some(50.0),
    };

    let result = engine.handle_transaction(deposit);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Account is locked");
}