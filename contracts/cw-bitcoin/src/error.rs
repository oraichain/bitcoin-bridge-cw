use bitcoin::blockdata::transaction::ParseOutPointError;
use cosmwasm_std::{OverflowError, StdError, VerificationError};
use std::env::VarError;

#[derive(thiserror::Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error(transparent)]
    Verify(#[from] VerificationError),
    #[error("Account Error: {0}")]
    Account(String),
    #[error("Coins Error: {0}")]
    Coins(String),
    #[error("Address Error: {0}")]
    Address(String),
    #[error(transparent)]
    Bitcoin(#[from] bitcoin::Error),
    #[error(transparent)]
    Overflow(#[from] OverflowError),
    #[error(transparent)]
    ParseOutPoint(#[from] ParseOutPointError),
    #[error(transparent)]
    BitcoinAddress(#[from] bitcoin::util::address::Error),
    #[error(transparent)]
    BitcoinHash(#[from] bitcoin::hashes::Error),
    #[error("{0}")]
    BitcoinPubkeyHash(String),
    #[error(transparent)]
    BitcoinLockTime(#[from] bitcoin::locktime::Error),
    #[error(transparent)]
    BitcoinEncode(#[from] bitcoin::consensus::encode::Error),
    #[error("Unable to deduct fee: {0}")]
    BitcoinFee(u64),
    #[error("{0}")]
    BitcoinRecoveryScript(String),
    #[error(transparent)]
    Bip32(#[from] bitcoin::util::bip32::Error),
    #[error("{0}")]
    Checkpoint(String),
    #[error(transparent)]
    Sighash(#[from] bitcoin::util::sighash::Error),
    #[error(transparent)]
    TryFrom(#[from] std::num::TryFromIntError),
    #[error("App Error: {0}")]
    App(String),
    #[error(transparent)]
    Secp(#[from] bitcoin::secp256k1::Error),
    #[error("Could not verify merkle proof")]
    BitcoinMerkleBlockError,
    #[error("{0}")]
    Header(String),
    #[error("{0}")]
    Ibc(String),
    #[error("Input index: {0} out of bounds")]
    InputIndexOutOfBounds(usize),
    #[error("{0}")]
    OutputError(String),
    #[error("Invalid Deposit Address")]
    InvalidDepositAddress,
    #[error("{0}")]
    Relayer(String),
    #[error("{0}")]
    Signer(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Warp Rejection")]
    WarpRejection(),
    #[error("{0}")]
    VarError(VarError),
    #[error("unauthorized")]
    Unauthorized {},
    #[error("Unknown Error")]
    Unknown,
}

impl From<ContractError> for StdError {
    fn from(source: ContractError) -> Self {
        Self::generic_err(source.to_string())
    }
}

pub type ContractResult<T> = std::result::Result<T, ContractError>;
