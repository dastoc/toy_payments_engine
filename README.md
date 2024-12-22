# Toy Payments Engine

## Overview

The **Toy Payments Engine** is a robust Rust-based application designed to process financial transactions from CSV files efficiently. It supports various transaction types such as deposits, withdrawals, disputes, and chargebacks. The engine ensures accuracy and scalability while handling both small and very large datasets.

---

## Project Structure
```bash
├── Cargo.lock
├── Cargo.toml
├── data
│   ├── large
│   │   ├── 7_clients.csv
│   │   ├── dataset_generator.rs
│   │   └── very_large_transactions.csv
│   └── small
│       ├── edge_case_transactions.csv
│       ├── invalid_transactions.csv
│       ├── sample_transactions.csv
│       ├── small_transaction.csv
│       ├── transactions.csv
│       └── valid_transactions.csv
├── src
│   ├── engine.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── models.rs
│   └── utils.rs
└── tests
├── engine_tests.rs
├── main_tests.rs
├── models_tests.rs
└── utils_tests.rs
```
---

## Features

### Core Functionalities

1. **Transaction Processing**
   - Supports `deposit`, `withdrawal`, `dispute`, `resolve`, and `chargeback`.
   - Handles large datasets with millions of transactions efficiently.

2. **Error Handling**
   - Logs errors for invalid transactions while skipping them.
   - Ensures disputes, resolves, and chargebacks reference existing transactions.

3. **CSV Export**
   - Outputs client account states in CSV format with high precision (four decimal places).

4. **Progress Tracking**
   - Real-time progress messages for processing large input files.

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [rust-script](https://crates.io/crates/rust-script) for dataset generation (optional)

### Running the Engine

To process transactions:
```bash
cargo run --release -- <input_file.csv> > <output_file.csv>
```

### Dataset Generation

The dataset_generator.rs script (located in data/large/) generates datasets of configurable sizes for testing scalability:
```bash
cargo install rust-script
rust-script data/large/dataset_generator.rs -- <num_clients> <transactions_per_client> <output_file>
```

### Input & Output

#### Input Format

The input CSV file should have the following columns:
- type: Transaction type (e.g., deposit, withdrawal, etc.).
- client: Client ID (u16).
- tx: Transaction ID (u32).
- amount: Optional transaction amount.

#### Output Format

The output CSV contains the following fields:
- client: Client ID.
- available: Funds available for transactions.
- held: Funds held due to disputes.
- total: Total funds (available + held).
- locked: Whether the account is locked.

### Supported Transaction Types
1.  **Deposit**
	Increases available and total funds.
2.	**Withdrawal**
	Decreases available and total funds if sufficient balance exists.
3.	**Dispute**
	Moves funds from available to held.
4.	**Resolve**
	Reverses a dispute, returning funds from held to available.
5.	**Chargeback**
	Finalizes a dispute, deducting funds from held and locking the account.

### Performance Features
1.	**Streaming Processing**
	Processes CSV rows incrementally to minimize memory usage.
2.	**Progress Updates**
	Displays progress every 10,000 transactions processed.
3.	**Memory Optimization**
	Efficient in-memory data handling for accounts and transactions.

### Testing

#### Unit Tests

Each module (engine.rs, models.rs, utils.rs) includes comprehensive unit tests.

#### Running Tests

Run all tests with:
```bash
cargo test
```

### Sample Datasets
- Small Datasets: Located in `data/small/` for basic testing.
- Large Datasets: Located in `data/large/` to test scalability.

### Assumptions
1.	Each client has a single account.
2.	Transactions reference valid u16 client IDs and u32 transaction IDs.
3.	Input files are UTF-8 encoded and well-formed CSVs.

### Future Enhancements
1.	**Parallel Processing**
	Enhance scalability for multi-core systems.
2.	**Improved Logging**
	Add more granular logs for debugging and monitoring.
3.	**Configurable Parameters**
	Allow users to configure progress update thresholds.

The Toy Payments Engine exemplifies clean, scalable, and maintainable Rust code. It effectively handles complex financial transactions, ensuring correctness and performance, even for large datasets.