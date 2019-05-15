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

//! Defines library's errors and internal types/utilies used in the protocol and
//! transactions

use std::convert::Into;

pub mod xmr;
pub mod btc;
pub mod common;
pub mod constants;

/// Library and dependencies' errors
#[derive(Debug)]
pub enum Error {
    /// Bitcoind feerate error
    CannotEstimateFeerate,
    /// Missing part in Bitcoin transaction
    TransactionNotComplete,
    /// One or more common parameters in the setup missmatch
    MissmatchCommonParameters,
    /// Missing value
    MissingValue,
    /// Invalid signature in Bitcoin transaction
    InvalidSignature,
    /// Bitcoin encoding/decoding error
    BitcoinConsensus(bitcoin::consensus::encode::Error),
    /// Signing library secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Random value generation error
    Rand(rand::Error),
    /// Serialization/deserialization error
    Serde(hex::FromHexError),
}

impl From<Error> for wasm_bindgen::JsValue {
    fn from(e: Error) -> wasm_bindgen::JsValue {
        format!("{:?}", e).into()
    }
}

impl From<rand::Error> for Error {
    fn from(e: rand::Error) -> Error {
        Error::Rand(e)
    }
}

impl From<std::option::NoneError> for Error {
    fn from(_: std::option::NoneError) -> Error {
        Error::MissingValue
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Error {
        Error::Secp256k1(e)
    }
}

impl From<bitcoin::consensus::encode::Error> for Error {
    fn from(e: bitcoin::consensus::encode::Error) -> Error {
        Error::BitcoinConsensus(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Error {
        Error::Serde(e)
    }
}

impl<T> Into<Result<T>> for Error {
    fn into(self) -> Result<T> {
        Err(self)
    }
}

/// Shortcut definition to handle results with library defined errors
pub type Result<T> = std::result::Result<T, Error>;

/// Define the two types of Locktime in a bitcoin transaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelativeLocktime {
    /// Based on block's timestamp
    Time(u16),
    /// Based on block's height
    Blocks(u16),
}

impl RelativeLocktime {
    /// Get the raw value used in nSequence
    pub fn as_u32(&self) -> u32 {
        use self::RelativeLocktime::*;

        match *self {
            Time(sec) => (0b1 << 22) | sec as u32,
            Blocks(block_number) => block_number as u32,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::{RelativeLocktime, common};

    #[test]
    fn relative_locktime_blocks() {
        let n_sequence = RelativeLocktime::Blocks(144);
        assert_eq!(144u32, n_sequence.as_u32());
    }

    #[test]
    fn relative_locktime_time() {
        let n_sequence = RelativeLocktime::Time(32);
        assert_eq!(0b00000000_01000000_00000000_00100000, n_sequence.as_u32());
    }

    #[test]
    fn new_common_param() {
        let t_0 = RelativeLocktime::Blocks(144);
        let t_1 = RelativeLocktime::Blocks(32);
        let params = common::Params::new(t_0, t_1);
        assert_eq!(144u32, params.t_0.as_u32());
        assert_eq!(32u32, params.t_1.as_u32());
    }
}
