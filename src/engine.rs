use std::collections::HashMap;

use crate::models::{ClientAccount, TransactionType, Transaction};

pub struct TransactionEngine {
    pub accounts: HashMap<u16, ClientAccount>,
    pub transactions: HashMap<u32, Transaction>,
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn handle_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        // Skip processing if the account is locked
        if let Some(account) = self.accounts.get_mut(&transaction.client_id) {
            if account.locked {
                return Err("Account is locked".into());
            }
        }

        match transaction.tx_type {
            TransactionType::Deposit => self.handle_deposit(transaction),
            TransactionType::Withdrawal => self.handle_withdrawal(transaction),
            TransactionType::Dispute => self.handle_dispute(transaction),
            TransactionType::Resolve => self.handle_resolve(transaction),
            TransactionType::Chargeback => self.handle_chargeback(transaction),
        }
    }

    fn handle_deposit(&mut self, transaction: Transaction) -> Result<(), String> {
        // Ensure the transaction has a valid amount
        let amount = transaction.amount.ok_or("Deposit transaction must have an amount")?;

        if amount <= 0.0 {
            return Err("Deposit amount must be positive".into());
        }

        // Get or create the client's account
        let account = self.accounts.entry(transaction.client_id).or_insert_with(|| ClientAccount {
            client_id: transaction.client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        });

        // Ensure the account is not locked
        if account.locked {
            return Err(format!(
                "Cannot deposit into a locked account (Client ID: {})",
                transaction.client_id
            ));
        }

        // Update account balances
        account.available += amount;
        account.total += amount;

        // Record the transaction
        self.transactions.insert(transaction.tx_id, transaction);

        Ok(())
    }

    fn handle_withdrawal(&mut self, transaction: Transaction) -> Result<(), String> {
        // Ensure the transaction has a valid amount
        let amount = transaction.amount.ok_or("Withdrawal transaction must have an amount")?;

        if amount <= 0.0 {
            return Err("Withdrawal amount must be positive".into());
        }

        // Fetch the client's account
        let account = self.accounts.get_mut(&transaction.client_id).ok_or_else(|| {
            format!(
                "Account for Client ID {} not found. Cannot process withdrawal.",
                transaction.client_id
            )
        })?;

        // Ensure the account is not locked
        if account.locked {
            return Err(format!(
                "Cannot withdraw from a locked account (Client ID: {})",
                transaction.client_id
            ));
        }

        // Ensure sufficient available funds
        if account.available < amount {
            return Err(format!(
                "Insufficient funds: Available = {}, Withdrawal amount = {}",
                account.available, amount
            ));
        }

        // Update account balances
        account.available -= amount;
        account.total -= amount;

        // Record the transaction
        self.transactions.insert(transaction.tx_id, transaction);

        Ok(())  
    }

    fn handle_dispute(&mut self, transaction: Transaction) -> Result<(), String> {
        // Fetch the referenced transaction
        let tx = self.transactions.get(&transaction.tx_id)
            .ok_or_else(|| format!("Transaction with ID {} not found for dispute", transaction.tx_id))?;

        // Validate client ID
        if tx.client_id != transaction.client_id {
            return Err(format!(
                "Dispute client ID mismatch: expected {}, got {}",
                tx.client_id, transaction.client_id
            ));
        }

        // Fetch the client's account
        let account = self.accounts.get_mut(&tx.client_id)
            .ok_or_else(|| format!("Account for client ID {} not found", tx.client_id))?;

        // Ensure the transaction has an amount
        let amount = tx.amount.ok_or_else(|| format!(
            "Transaction with ID {} does not have an associated amount", transaction.tx_id
        ))?;

        // Update account balances
        if account.available < amount {
            return Err(format!(
                "Insufficient funds: Available = {}, Dispute amount = {}",
                account.available, amount
            ));
        }

        account.available -= amount;
        account.held += amount;

        Ok(())
    }

    fn handle_resolve(&mut self, transaction: Transaction) -> Result<(), String> {
        // Fetch the referenced transaction
        let tx = self.transactions.get(&transaction.tx_id)
            .ok_or_else(|| format!("Transaction with ID {} not found for resolve", transaction.tx_id))?;

        // Validate client ID
        if tx.client_id != transaction.client_id {
            return Err(format!(
                "Resolve client ID mismatch: expected {}, got {}",
                tx.client_id, transaction.client_id
            ));
        }

        // Fetch the client's account
        let account = self.accounts.get_mut(&tx.client_id)
            .ok_or_else(|| format!("Account for client ID {} not found", tx.client_id))?;

        // Ensure the transaction has an amount
        let amount = tx.amount.ok_or_else(|| format!(
            "Transaction with ID {} does not have an associated amount", transaction.tx_id
        ))?;

        // Ensure sufficient held funds
        if account.held < amount {
            return Err(format!(
                "Insufficient held funds: Held = {}, Resolve amount = {}",
                account.held, amount
            ));
        }

        account.held -= amount;
        account.available += amount;

        Ok(())
    }

    fn handle_chargeback(&mut self, transaction: Transaction) -> Result<(), String> {
        // Fetch the referenced transaction
        let tx = self.transactions.get(&transaction.tx_id)
            .ok_or_else(|| format!("Transaction with ID {} not found for chargeback", transaction.tx_id))?;

        // Validate client ID
        if tx.client_id != transaction.client_id {
            return Err(format!(
                "Chargeback client ID mismatch: expected {}, got {}",
                tx.client_id, transaction.client_id
            ));
        }

        // Fetch the client's account
        let account = self.accounts.get_mut(&tx.client_id)
            .ok_or_else(|| format!("Account for client ID {} not found", tx.client_id))?;

        // Ensure the transaction has an amount
        let amount = tx.amount.ok_or_else(|| format!(
            "Transaction with ID {} does not have an associated amount", transaction.tx_id
        ))?;

        // Ensure sufficient held funds
        if account.held < amount {
            return Err(format!(
                "Insufficient held funds: Held = {}, Chargeback amount = {}",
                account.held, amount
            ));
        }

        // Update account balances and lock the account
        account.held -= amount;
        account.total -= amount;
        account.locked = true;

        Ok(())
    }
}