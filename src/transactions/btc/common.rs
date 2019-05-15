// Monero Swap Rust Library
// Written in 2019 by
//   h4sh3d <h4sh3d@truelevel.io>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//

//! Common structures between some transactions

use crate::types::{btc, xmr};
use crate::transactions::Transaction;

use secp256k1::Signature;
use bitcoin::blockdata::script::Script;
use bitcoin::util::address::Address;

/// Generate a new transaction base on previous outputs and one output to an address
pub struct New<'a, T: Transaction> {
    /// Previous transaction for building inputs
    pub(crate) prev_tx: &'a T,
    /// Final destination for building output
    pub(crate) final_address: Address,
}

/// Signing step for a transaction
pub struct Sign<'a, T: Transaction> {
    /// Private key used to sign
    pub(crate) privkey: &'a btc::PrivateKey,
    /// The script to sign
    pub(crate) script: &'a Script,
    /// The previous transaction
    pub(crate) prev_tx: &'a T,
}

/// Finalizing step for a transaction with a signature, a script, a Monero private key
/// revealed in the script, and a secret value for hash locks
pub struct Finalize<'a> {
    pub(crate) sig: Signature,
    pub(crate) script: Script,
    pub(crate) privkey: Option<&'a xmr::PrivateKey>,
    pub(crate) secret: Option<[u8; 32]>,
}
