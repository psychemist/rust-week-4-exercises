use std::{io::Read, str::FromStr};
use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        // Implement constructor for Point
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        // Return a new builder for constructing a transaction
        LegacyTransactionBuilder::default()
    }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        // Implement default values
        Self {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        // Initialize new builder by calling default
        Self::default()
    }

    pub fn version(mut self, version: i32) -> Self {
        // Set the transaction version
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        // Add input to the transaction
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        // Add output to the transaction
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        // Set lock_time for transaction
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        // Build and return the final LegacyTransaction
        LegacyTransaction {
            version: self.version,
            inputs: self.inputs,
            outputs: self.outputs,
            lock_time: self.lock_time,
        }
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8> {
        // Implement serialization to bytes
        vec![]
    }
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        // Serialize only version and lock_time (simplified)
        let mut serialized_tx = Vec::<u8>::with_capacity(8);
        serialized_tx.extend(self.version.to_le_bytes());
        serialized_tx.extend(self.lock_time.to_le_bytes());
        serialized_tx
    }
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // Parse binary data into a LegacyTransaction
        let mut data = data;
        
        // Minimum length is 12 bytes (4 version + 4 inputs count + 4 lock_time)
        if data.len() < 12 {
            Err(BitcoinError::InvalidTransaction)
        } else {
            // Read tx fields from data input and build LegacyTransaction
            let mut version_buf = [0; 4];
            let mut input_buf = [0; 4];
            let mut ouput_buf = [0; 4];
            let mut lock_time_buf = [0; 4];

            let _ = data.read_exact(&mut version_buf);
            let _ = data.read_exact(&mut input_buf);
            let _ = data.read_exact(&mut ouput_buf);
            let _ = data.read_exact(&mut lock_time_buf);

            let input_count = u32::from_le_bytes(input_buf);
            let output_count = u32::from_le_bytes(ouput_buf);

            Ok(LegacyTransaction {
                version: i32::from_le_bytes(version_buf),
                inputs: Vec::with_capacity(input_count as usize),
                outputs: Vec::with_capacity(output_count as usize),
                lock_time: u32::from_le_bytes(lock_time_buf),
            })
        }
    }
}

// // Simple CLI argument parser
// pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
//     // TODO: Match args to "send" or "balance" commands and parse required arguments
// }

// pub enum CliCommand {
//     Send { amount: u64, address: String },
//     Balance,
// }
