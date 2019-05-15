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

//! Collections of structures to create and validate transactions

use bitcoin::consensus::encode::deserialize;
use crate::types::Result;

pub mod btc;
//pub mod xmr;

/// Represente a transaction that can be send and receive over the network
pub trait Transaction {
    fn to_hex(&self) -> Option<String>;

    fn to_transaction(&self) -> Option<bitcoin::Transaction> {
        match self.to_hex() {
            Some(tx) => match hex::decode(&tx) {
                Ok(res) => match deserialize(&res[..]) {
                    Ok(res) => Some(res),
                    Err(_) => None,
                },
                Err(_) => None,
            },
            None => None,
        }
    }
}

/// Transaction have building steps
pub trait Builder<T> {
    type Ret;

    fn build(&mut self, params: T) -> Result<Self::Ret>;
}

/// Transaction have validating steps
pub trait Validator<T> {
    fn validate(&self, params: T) -> Result<()>;
}
