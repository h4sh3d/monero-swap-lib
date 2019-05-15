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

use crate::transactions;
use crate::types::{xmr, RelativeLocktime};

use secp256k1::Signature;
use bitcoin_hashes::sha256d;
use bitcoin::Address;
use curve25519_dalek::constants;
use wasm_bindgen::prelude::*;

pub mod scripts;

pub type PrivateKey = secp256k1::SecretKey;

pub type PublicKey = secp256k1::PublicKey;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Utxo {
    pub txid: sha256d::Hash,
    pub vout: u32,
    pub amount: u64,
}

pub struct Input<'a> {
    pub amount: u64,
    pub address: &'a Address,
    pub privkey: &'a PrivateKey,
}

pub struct CreateTransactions {
    pub utxo: Utxo,
}

#[derive(Debug, Clone)]
pub struct InitialTransactions {
    pub btx_1: String,
    pub btx_2: String,
    pub sig_b: Signature,
}

pub struct LockFunds<'a> {
    pub input: Input<'a>,
    pub btx_1: String,
    pub btx_2_signed: String,
}

pub struct SpendRefund<'a> {
    pub btx_2_signed: &'a transactions::btc::refund::RefundTx<'a>,
    pub address: Address,
}

pub struct VerifyXmrLock;

pub struct ReleaseXmr;

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SetupParams {
    pub(crate) a_1: xmr::PrivateKey,
    pub(crate) x_1: xmr::PrivateKey,
    pub(crate) b_b: PrivateKey,
    pub(crate) B_b: PublicKey,
    pub(crate) s: [u8; 32],
    pub(crate) h_1: [u8; 32],
    pub(crate) h_2: [u8; 32],
    pub(crate) t_0: RelativeLocktime,
    pub(crate) t_1: RelativeLocktime,
}

#[wasm_bindgen(js_name = __wbg_btcexportedsetupparams_free)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ExportedSetupParams {
    pub(crate) a_1: xmr::PrivateKey,
    pub(crate) X_1: xmr::PublicKey,
    pub(crate) B_b: PublicKey,
    pub(crate) h_1: [u8; 32],
    pub(crate) h_2: [u8; 32],
    pub(crate) t_0: RelativeLocktime,
    pub(crate) t_1: RelativeLocktime,
}

impl From<&SetupParams> for ExportedSetupParams {
    #[allow(non_snake_case)]
    fn from(params: &SetupParams) -> ExportedSetupParams {
        let SetupParams {
            a_1,
            x_1,
            b_b: _,
            B_b,
            s: _,
            h_1,
            h_2,
            t_0,
            t_1,
        } = params;

        let X_1 = x_1 * &constants::ED25519_BASEPOINT_TABLE;

        ExportedSetupParams {
            a_1: *a_1,
            X_1,
            B_b: B_b.clone(),
            h_1: *h_1,
            h_2: *h_2,
            t_0: *t_0,
            t_1: *t_1,
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Setup {
    pub(crate) a: xmr::PrivateKey,
    pub(crate) x_1: xmr::PrivateKey,
    pub(crate) X: xmr::PublicKey,
    pub(crate) B_a: PublicKey,
    pub(crate) b_b: PrivateKey,
    pub(crate) B_b: PublicKey,
    pub(crate) s: [u8; 32],
    pub(crate) h_0: [u8; 32],
    pub(crate) h_1: [u8; 32],
    pub(crate) h_2: [u8; 32],
    pub(crate) t_0: RelativeLocktime,
    pub(crate) t_1: RelativeLocktime,
}

impl Setup {
    pub fn get_s(&self) -> [u8; 32] {
        self.s
    }
}
