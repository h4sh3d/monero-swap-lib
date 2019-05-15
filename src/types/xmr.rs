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

use bitcoin::Address;
use curve25519_dalek::constants;
use wasm_bindgen::prelude::*;

use super::{btc, RelativeLocktime};
use crate::transactions;

pub type PrivateKey = curve25519_dalek::scalar::Scalar;

pub type PublicKey = curve25519_dalek::edwards::EdwardsPoint;

pub struct VerifyTransactions {
    pub transactions: btc::InitialTransactions,
}

#[derive(Debug)]
pub struct VerifiedTransaction {
    pub btx_2_signed: String,
}

pub struct InitiateSwap;

pub struct Swap {
    pub funding: transactions::btc::funding::FundingTx,
    pub address: Address,
    pub s: [u8; 32],
}

pub struct ClaimRefund<'a> {
    pub btx_2_signed: &'a transactions::btc::refund::RefundTx<'a>,
    pub address: Address,
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SetupParams {
    pub(crate) a_0: PrivateKey,
    pub(crate) x_0: PrivateKey,
    pub(crate) b_a: btc::PrivateKey,
    pub(crate) B_a: btc::PublicKey,
    pub(crate) h_0: [u8; 32],
    pub(crate) t_0: RelativeLocktime,
    pub(crate) t_1: RelativeLocktime,
}

#[wasm_bindgen]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ExportedSetupParams {
    pub(crate) a_0: PrivateKey,
    pub(crate) X_0: PublicKey,
    pub(crate) B_a: btc::PublicKey,
    pub(crate) h_0: [u8; 32],
    pub(crate) t_0: RelativeLocktime,
    pub(crate) t_1: RelativeLocktime,
}

impl From<&SetupParams> for ExportedSetupParams {
    #[allow(non_snake_case)]
    fn from(params: &SetupParams) -> ExportedSetupParams {
        let SetupParams {
            a_0,
            x_0,
            b_a: _,
            B_a,
            h_0,
            t_0,
            t_1,
        } = params;

        let X_0 = x_0 * &constants::ED25519_BASEPOINT_TABLE;

        ExportedSetupParams {
            a_0: *a_0,
            X_0,
            B_a: B_a.clone(),
            h_0: *h_0,
            t_0: *t_0,
            t_1: *t_1,
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Setup {
    pub(crate) a: PrivateKey,
    pub(crate) x_0: PrivateKey,
    pub(crate) X: PublicKey,
    pub(crate) b_a: btc::PrivateKey,
    pub(crate) B_a: btc::PublicKey,
    pub(crate) B_b: btc::PublicKey,
    pub(crate) h_0: [u8; 32],
    pub(crate) h_1: [u8; 32],
    pub(crate) h_2: [u8; 32],
    pub(crate) t_0: RelativeLocktime,
    pub(crate) t_1: RelativeLocktime,
}
