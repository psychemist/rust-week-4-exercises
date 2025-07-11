use clap::{Parser, Subcommand};
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

impl<T: FromStr> FromStr for Point<T> {
    type Err = BitcoinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(BitcoinError::ParseError("Error parsing values".to_string()))?;

        let x_fromstr = x
            .parse::<T>()
            .map_err(|_| BitcoinError::ParseError("Could not parse x value".to_string()))?;
        let y_fromstr = y
            .parse::<T>()
            .map_err(|_| BitcoinError::ParseError("Could not parse y value".to_string()))?;

        Ok(Point {
            x: x_fromstr,
            y: y_fromstr,
        })
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

#[derive(Parser)]
#[command(name = "BTxC Decoder")]
#[command(version = "1.0.0")]
#[command(about = "Bitcoin Transaction Decoder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Sends bitcoin of { amount } to recipient { address }
    Send {
        #[arg(
            required = true,
            help = "(numeric, required) The amount of bitcoin you want to send in satoshis"
        )]
        amount: u64,
        #[arg(
            required = true,
            help = "(string, required) The address of the recipient you want to send bitcoins to"
        )]
        address: String,
    },

    /// Returns the balance of transaction sender
    Balance,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    // Match args to "send" or "balance" commands and parse required arguments
    if args.len() == 0 {
        return Err(BitcoinError::ParseError(String::from(
            "No arguments provided",
        )));
    }

    let mut command: Vec<String> = vec![];
    command.push("BTxC Decoder".to_string());
    for i in 0..args.len() {
        command.push(args[i].clone());
    }

    let cli = match Cli::try_parse_from(command) {
        Ok(cli) => cli,
        Err(_) => {
            return Err(BitcoinError::ParseError(
                "Failed to parse arguments".to_string(),
            ));
        }
    };

    match &cli.command {
        Some(CliCommand::Send { amount, address }) => {
            if Some(amount).is_none() {
                return Err(BitcoinError::ParseError("Amount is required".to_string()));
            } else if address.is_empty() {
                return Err(BitcoinError::ParseError(
                    "Address cannot be empty".to_string(),
                ));
            } else if *amount == 0 {
                return Err(BitcoinError::InvalidAmount);
            } else {
                println!("Sending {} satoshis to {}!", amount, address);
                return Ok(CliCommand::Send {
                    amount: *amount,
                    address: address.clone(),
                });
            }
        }
        Some(CliCommand::Balance) => {
            return Ok(CliCommand::Balance);
        }
        _ => Err(BitcoinError::ParseError(String::from(
            "No valid command specified",
        ))),
    }
}
