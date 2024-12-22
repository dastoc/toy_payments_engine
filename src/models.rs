use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

//Enum for transaction types, ensuring type safety
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl FromStr for TransactionType {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "deposit" => Ok(TransactionType::Deposit),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            "dispute" => Ok(TransactionType::Dispute),
            "resolve" => Ok(TransactionType::Resolve),
            "chargeback" => Ok(TransactionType::Chargeback),
            _ => Err("Invalid transaction type"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(deserialize_with = "deserialize_option_float_with_precision")]
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ClientAccount {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(serialize_with = "serialize_float_with_precision")]
    pub available: f64,
    #[serde(serialize_with = "serialize_float_with_precision")]
    pub held: f64,
    #[serde(serialize_with = "serialize_float_with_precision")]
    pub total: f64,
    pub locked: bool,
}

// Serializes a floating-point value with the required precision.
fn serialize_float_with_precision<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{:.4}", value))
}

// Deserializes a floating-point value with the required precision.
fn deserialize_option_float_with_precision<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<f64> = Option::deserialize(deserializer)?;
    
    if let Some(v) = value {
        let factor = 10f64.powi(4);
        let rounded = (v * factor).round() / factor;
        Ok(Some(rounded))
    } else {
        Ok(None)
    }
}