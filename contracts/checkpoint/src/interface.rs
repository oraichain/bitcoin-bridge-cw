use std::ops::Deref;

use bitcoin::util::bip32::ExtendedPubKey;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Coin;
use serde::{Deserialize, Serialize};
// use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};

use crate::constants::MAX_CHECKPOINT_AGE;
use crate::constants::MAX_CHECKPOINT_INTERVAL;
use crate::constants::MAX_FEE_RATE;
use crate::constants::MIN_FEE_RATE;
use crate::constants::USER_FEE_FACTOR;
use crate::error::ContractResult;
use crate::signatory::SIGSET_THRESHOLD;

// pub trait DequeExtension<'a, T: Serialize + DeserializeOwned> {
//     fn retain_unordered<F>(&self, store: &mut dyn Storage, f: F) -> StdResult<u64>
//     where
//         F: FnMut(&T) -> bool;
// }

// impl<'a, T: Serialize + DeserializeOwned> DequeExtension<'a, T> for Deque<'a, T> {
//     fn retain_unordered<F>(&self, store: &mut dyn Storage, mut f: F) -> StdResult<u64>
//     where
//         F: FnMut(&T) -> bool,
//     {
//         let mut temp = vec![];
//         while let Some(item) = self.pop_front(store)? {
//             temp.push(item);
//         }
//         let mut size = 0;
//         for item in temp {
//             if f(&item) {
//                 self.push_back(store, &item)?;
//                 size += 1;
//             }
//         }

//         Ok(size)
//     }
// }

#[cw_serde]
pub struct Accounts {
    transfers_allowed: bool,
    transfer_exceptions: Vec<String>,
    accounts: Vec<(String, Coin)>,
}

impl Accounts {
    pub fn balance(&self, address: String) -> Option<Coin> {
        self.accounts
            .iter()
            .find(|item| item.0 == address)
            .map(|item| item.1.clone())
    }
}

#[cw_serde]
pub struct IbcDest {
    pub source_port: String,
    pub source_channel: String,
    #[serde(skip)]
    pub receiver: String,
    #[serde(skip)]
    pub sender: String,
    pub timeout_timestamp: u64,
    pub memo: String,
}

#[cw_serde]
pub enum Dest {
    Address(Addr),
    Ibc(IbcDest),
}

impl Dest {
    pub fn to_receiver_addr(&self) -> String {
        match self {
            Self::Address(addr) => addr.to_string(),
            Self::Ibc(dest) => dest.receiver.to_string(),
        }
    }

    pub fn commitment_bytes(&self) -> ContractResult<Vec<u8>> {
        let bytes = match self {
            Self::Address(addr) => addr.as_bytes().into(),
            Self::Ibc(dest) => Sha256::digest(dest.receiver.as_bytes()).to_vec(),
        };

        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Validator {
    pub pubkey: Vec<u8>,
    pub power: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BitcoinConfig {
    /// The minimum number of checkpoints that must be produced before
    /// withdrawals are enabled.
    pub min_withdrawal_checkpoints: u32,
    /// The minimum amount of BTC a deposit must send to be honored, in
    /// satoshis.
    pub min_deposit_amount: u64,
    /// The minimum amount of BTC a withdrawal must withdraw, in satoshis.
    pub min_withdrawal_amount: u64,
    /// TODO: remove this, not used
    pub max_withdrawal_amount: u64,
    /// The maximum length of a withdrawal output script, in bytes.
    pub max_withdrawal_script_length: u64,
    /// The fee charged for an nBTC transfer, in micro-satoshis.
    pub transfer_fee: u64,
    /// The minimum number of confirmations a Bitcoin block must have before it
    /// is considered finalized. Note that in the current implementation, the
    /// actual number of confirmations required is `min_confirmations + 1`.
    pub min_confirmations: u32,
    /// The number which amounts in satoshis are multiplied by to get the number
    /// of units held in nBTC accounts. In other words, the amount of
    /// subdivisions of satoshis which nBTC accounting uses.
    pub units_per_sat: u64,

    // (These fields were moved to `checkpoint::Config`)
    pub emergency_disbursal_min_tx_amt: u64,

    pub emergency_disbursal_lock_time_interval: u32,

    pub emergency_disbursal_max_tx_size: u64,

    /// If a signer does not submit signatures for this many consecutive
    /// checkpoints, they are considered offline and are removed from the
    /// signatory set (jailed) and slashed.    
    pub max_offline_checkpoints: u32,
    /// The minimum number of confirmations a checkpoint must have on the
    /// Bitcoin network before it is considered confirmed. Note that in the
    /// current implementation, the actual number of confirmations required is
    /// `min_checkpoint_confirmations + 1`.    
    pub min_checkpoint_confirmations: u32,
    /// The maximum amount of BTC that can be held in the network, in satoshis.    
    pub capacity_limit: u64,

    pub max_deposit_age: u64,

    pub fee_pool_target_balance: u64,

    pub fee_pool_reward_split: (u64, u64),
}

/// Configuration parameters used in processing checkpoints.
#[cw_serde]
pub struct CheckpointConfig {
    /// The minimum amount of time between the creation of checkpoints, in
    /// seconds.
    ///
    /// If a checkpoint is to be created, but less than this time has passed
    /// since the last checkpoint was created (in the `Building` state), the
    /// current `Building` checkpoint will be delayed in advancing to `Signing`.
    pub min_checkpoint_interval: u64,

    /// The maximum amount of time between the creation of checkpoints, in
    /// seconds.
    ///
    /// If a checkpoint would otherwise not be created, but this amount of time
    /// has passed since the last checkpoint was created (in the `Building`
    /// state), the current `Building` checkpoint will be advanced to `Signing`
    /// and a new `Building` checkpoint will be added.
    pub max_checkpoint_interval: u64,

    /// The maximum number of inputs allowed in a checkpoint transaction.
    ///
    /// This is used to prevent the checkpoint transaction from being too large
    /// to be accepted by the Bitcoin network.
    ///
    /// If a checkpoint has more inputs than this when advancing from `Building`
    /// to `Signing`, the excess inputs will be moved to the suceeding,
    /// newly-created `Building` checkpoint.
    pub max_inputs: u64,

    /// The maximum number of outputs allowed in a checkpoint transaction.
    ///
    /// This is used to prevent the checkpoint transaction from being too large
    /// to be accepted by the Bitcoin network.
    ///
    /// If a checkpoint has more outputs than this when advancing from `Building`
    /// to `Signing`, the excess outputs will be moved to the suceeding,
    /// newly-created `Building` checkpoint.∑
    pub max_outputs: u64,

    /// The default fee rate to use when creating the first checkpoint of the
    /// network, in satoshis per virtual byte.    
    pub fee_rate: u64,

    /// The maximum age of a checkpoint to retain, in seconds.
    ///
    /// Checkpoints older than this will be pruned from the state, down to a
    /// minimum of 10 checkpoints in the checkpoint queue.
    pub max_age: u64,

    /// The number of blocks to target for confirmation of the checkpoint
    /// transaction.
    ///
    /// This is used to adjust the fee rate of the checkpoint transaction, to
    /// ensure it is confirmed within the target number of blocks. The fee rate
    /// will be adjusted up if the checkpoint transaction is not confirmed
    /// within the target number of blocks, and will be adjusted down if the
    /// checkpoint transaction faster than the target.    
    pub target_checkpoint_inclusion: u32,

    /// The lower bound to use when adjusting the fee rate of the checkpoint
    /// transaction, in satoshis per virtual byte.    
    pub min_fee_rate: u64,

    /// The upper bound to use when adjusting the fee rate of the checkpoint
    /// transaction, in satoshis per virtual byte.    
    pub max_fee_rate: u64,

    /// The value (in basis points) to multiply by when calculating the miner
    /// fee to deduct from a user's deposit or withdrawal. This value should be
    /// at least 1 (10,000 basis points).
    ///
    /// The difference in the fee deducted and the fee paid in the checkpoint
    /// transaction is added to the fee pool, to help the network pay for
    /// its own miner fees.    
    pub user_fee_factor: u64,

    /// The threshold of signatures required to spend reserve scripts, as a
    /// ratio represented by a tuple, `(numerator, denominator)`.
    ///
    /// For example, `(9, 10)` means the threshold is 90% of the signatory set.    
    pub sigset_threshold: (u64, u64),

    /// The minimum amount of nBTC an account must hold to be eligible for an
    /// output in the emergency disbursal.    
    pub emergency_disbursal_min_tx_amt: u64,

    /// The amount of time between the creation of a checkpoint and when the
    /// associated emergency disbursal transactions can be spent, in seconds.    
    pub emergency_disbursal_lock_time_interval: u32,

    /// The maximum size of a final emergency disbursal transaction, in virtual
    /// bytes.
    ///
    /// The outputs to be included in final emergency disbursal transactions
    /// will be distributed across multiple transactions around this size.    
    pub emergency_disbursal_max_tx_size: u64,

    /// The maximum number of unconfirmed checkpoints before the network will
    /// stop creating new checkpoints.
    ///
    /// If there is a long chain of unconfirmed checkpoints, there is possibly
    /// an issue causing the transactions to not be included on Bitcoin (e.g. an
    /// invalid transaction was created, the fee rate is too low even after
    /// adjustments, Bitcoin miners are censoring the transactions, etc.), in
    /// which case the network should evaluate and fix the issue before creating
    /// more checkpoints.
    ///
    /// This will also stop the fee rate from being adjusted too high if the
    /// issue is simply with relayers failing to report the confirmation of the
    /// checkpoint transactions.    
    pub max_unconfirmed_checkpoints: u32,
}

impl CheckpointConfig {
    fn bitcoin() -> Self {
        Self {
            min_checkpoint_interval: 60 * 5,
            max_checkpoint_interval: MAX_CHECKPOINT_INTERVAL,
            max_inputs: 40,
            max_outputs: 200,
            max_age: MAX_CHECKPOINT_AGE,
            target_checkpoint_inclusion: 2,
            min_fee_rate: MIN_FEE_RATE, // relay threshold is 1 sat/vbyte
            max_fee_rate: MAX_FEE_RATE,
            user_fee_factor: USER_FEE_FACTOR, // 2.7x
            sigset_threshold: SIGSET_THRESHOLD,
            emergency_disbursal_min_tx_amt: 1000,
            emergency_disbursal_lock_time_interval: 60 * 60 * 24 * 7 * 8, // 8 weeks
            emergency_disbursal_max_tx_size: 50_000,
            max_unconfirmed_checkpoints: 15,
            fee_rate: 0,
        }
    }
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self::bitcoin()
    }
}

/// A Bitcoin extended public key, used to derive Bitcoin public keys which
/// signatories sign transactions with.
// #[derive(Call, Query, Clone, Debug, Client, PartialEq, Serialize)]
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct Xpub {
    key: ExtendedPubKey,
}

impl Xpub {
    /// Creates a new `Xpub` from an `ExtendedPubKey`.
    pub fn new(key: ExtendedPubKey) -> Self {
        Xpub { key }
    }

    /// Gets the `ExtendedPubKey` from the `Xpub`.
    pub fn inner(&self) -> &ExtendedPubKey {
        &self.key
    }
}

impl Deref for Xpub {
    type Target = ExtendedPubKey;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}